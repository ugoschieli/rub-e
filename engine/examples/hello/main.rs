use egui::Ui;
use etib::camera::Camera;
use etib::core::bind_group::{BindGroupLayoutBuilder, FrameBuffered};
use etib::core::shaders::ShaderBuilder;
use etib::core::surface::Frame;
use etib::core::texture::Texture;
use etib::game::{EngineContext, Game};
use etib::renderer::{Renderer, SHADER_DIR};
use glam::{USizeVec3, usizevec3};
use noise::{NoiseFn, Perlin};
use winit::event::DeviceEvent;
use winit::window::CursorGrabMode;

pub mod updater;

/// Voxel counts per axis.
const GRID_SIZE: USizeVec3 = usizevec3(1024 * 2, 1024 * 2, 32);

/// Edge length of one (cubic) voxel in world units, i.e. meters.
const VOXEL_SIZE: f32 = 0.01;

/// Grid parameters uploaded to the DDA shader. The CPU owns these values so the
/// shader can never drift out of sync. Layout matches `Grid` in `dda.wesl`: a
/// `vec3<u32>` (16-byte aligned) with the trailing `f32` packed at offset 12,
/// for 16 bytes total.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GridUniform {
    size: [u32; 3],
    voxel_size: f32,
}

/// Build the voxel occupancy grid on the CPU from 3D Perlin noise, packed one
/// bit per voxel into u32 words in x-major order (matching `is_solid` in the
/// shader: `index = x + y*size + z*size*size`).
fn generate_occupancy(ctx: &EngineContext) -> Vec<u32> {
    // Spacing between voxels in noise space; smaller -> larger, smoother blobs.
    const FREQUENCY: f64 = 0.18;
    // Perlin output is roughly [-1, 1]; a positive cutoff keeps the field sparse.
    const THRESHOLD: f64 = 0.1;

    let perlin = Perlin::new(ctx.time.frame_number as u32);
    let total = GRID_SIZE.x * GRID_SIZE.y * GRID_SIZE.z;
    let mut words = vec![0u32; total.div_ceil(32)];

    for z in 0..GRID_SIZE.z {
        for y in 0..GRID_SIZE.y {
            for x in 0..GRID_SIZE.x {
                let sample = perlin.get([
                    x as f64 * FREQUENCY,
                    y as f64 * FREQUENCY,
                    z as f64 * FREQUENCY,
                ]);
                if sample > THRESHOLD {
                    let index = x + y * GRID_SIZE.x + z * GRID_SIZE.x * GRID_SIZE.y;
                    words[index / 32] |= 1u32 << (index % 32);
                }
            }
        }
    }

    words
}

struct Hello {
    camera: Camera,
    dda_renderer: DdaRenderer,
}

#[derive(Debug)]
struct DdaRenderer {
    output_texture: Texture,
    // Static voxel occupancy, shared by every frame's bind group.
    // occupancy_buffer: wgpu::Buffer,
    bind_group: FrameBuffered,
    pipeline: wgpu::ComputePipeline,
}

impl DdaRenderer {
    fn new(ctx: &EngineContext, camera: &Camera) -> Self {
        let gfx = &ctx.gfx;

        let output_texture = gfx.create_texture_2d(
            "dda_output_texture",
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            gfx.surface.config.width,
            gfx.surface.config.height,
        );

        let shader = ShaderBuilder::new(SHADER_DIR)
            .build_wgsl(&gfx.device, "package::dda")
            .unwrap();

        let occupancy_buffer = etib::utils::create_buffer_init(
            &gfx.device,
            "dda_occupancy",
            wgpu::BufferUsages::STORAGE,
            &generate_occupancy(ctx),
        );

        let grid_buffer = etib::utils::create_buffer_init(
            &gfx.device,
            "dda_grid",
            wgpu::BufferUsages::UNIFORM,
            &[GridUniform {
                size: [GRID_SIZE.x as u32, GRID_SIZE.y as u32, GRID_SIZE.z as u32],
                voxel_size: VOXEL_SIZE,
            }],
        );

        let bind_group_layout = BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::COMPUTE)
            .storage_texture_2d(
                0,
                wgpu::StorageTextureAccess::WriteOnly,
                output_texture.texture.format(),
            )
            .uniform(1) // Camera
            .storage(2, true) // Voxel occupancy
            .uniform(3) // Grid dimensions + voxel size
            .build();

        let bind_group = FrameBuffered::new(&gfx.device, bind_group_layout, |i| {
            vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&output_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera.buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: occupancy_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: grid_buffer.as_entire_binding(),
                },
            ]
        });

        let pipeline =
            gfx.create_compute_pipeline("dda_compute_pass", Some(&bind_group.layout), 0, &shader);
        Self {
            output_texture,
            // occupancy_buffer,
            bind_group,
            pipeline,
        }
    }
}

const WORKGROUP_SIZE_X: u32 = 8;
const WORKGROUP_SIZE_Y: u32 = 8;

impl Renderer for DdaRenderer {
    fn render(&mut self, ctx: &mut EngineContext, frame: &mut Frame) {
        {
            let mut dda_pass = ctx.gfx.create_compute_pass("dda_pass", &mut frame.encoder);
            dda_pass.set_pipeline(&self.pipeline);
            dda_pass.set_bind_group(0, self.bind_group.current(ctx.gfx.frame_index), &[]);

            let x = self
                .output_texture
                .texture
                .width()
                .div_ceil(WORKGROUP_SIZE_X);
            let y = self
                .output_texture
                .texture
                .height()
                .div_ceil(WORKGROUP_SIZE_Y);
            dda_pass.dispatch_workgroups(x, y, 1);
        }

        frame.encoder.copy_texture_to_texture(
            self.output_texture.texture.as_image_copy(),
            frame.surface_texture.texture.as_image_copy(),
            wgpu::Extent3d {
                width: frame.surface_texture.texture.width(),
                height: frame.surface_texture.texture.height(),
                depth_or_array_layers: 1,
            },
        );
    }
}

impl Game for Hello {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _params: Self::InitParams) -> Self {
        ctx.set_cursor_visible(false);
        let _ = ctx.set_cursor_grab(CursorGrabMode::Locked);

        let camera = Camera::new(&ctx.gfx);
        let dda_renderer = DdaRenderer::new(ctx, &camera);

        Self {
            camera,
            dda_renderer,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        self.camera.handle_keyboard(&ctx.input, &ctx.time);

        self.camera.upload(&ctx.gfx, ctx.window_size(), &ctx.time);
    }

    fn render(&mut self, ctx: &mut EngineContext, frame: &mut Frame) {
        self.dda_renderer.render(ctx, frame);
    }

    fn ui(&mut self, ui: &mut Ui) {
        let rect = ui.max_rect();
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Hello, ETIB!",
            egui::FontId::proportional(48.0),
            egui::Color32::from_rgb(255, 128, 0),
        );
    }

    fn device_input(&mut self, _ctx: &mut EngineContext, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.camera.handle_mouse(*delta);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    etib::game::run::<Hello>(None, Some(()))?;

    Ok(())
}
