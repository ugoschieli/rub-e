//! High-level scene renderer: static models, dynamic models, skybox, and HDR.
//!
//! [`Scene`] is the single rendering entry point. It owns the render pipeline,
//! GPU buffers, frustum culling, dynamic model management, the HDR intermediate
//! texture, and the optional skybox. The caller only needs to load models,
//! update transforms, and call [`Scene::render`] each frame.

use anyhow::Result;
use cgmath::{Matrix4, Vector4};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::cube::{
    Cube, CubeRaw, CullingPass, DynamicModel, DynamicScene, INDICES, ModelCube, VERTICES,
};
use crate::gfx::Gfx;
use crate::hdr::{HdrLoader, HdrPipeline, TonemappingMode};
use crate::{EngineContext, Vertex};
use etib_core::bindgroup::BindGroupBuilder;
use etib_core::pipeline::Pipeline;
use etib_core::shader::Shader;

const CUBE_SHADER: &str = include_str!("shaders/shader.wgsl");
const SKYBOX_SHADER: &str = include_str!("shaders/skybox.wgsl");

struct SkyboxData {
    pipeline: Pipeline,
    bind_group: etib_core::bindgroup::BindGroup,
}

/// Manages the full rendering of static models, dynamic models, skybox, and HDR.
///
/// Static models are baked in at construction time and never change. Dynamic
/// models can be added, removed, and transformed at runtime — each is a
/// multi-cube group from a `.model` file that moves as a rigid body.
///
/// # Frame loop
///
/// ```text
/// // A single call handles culling, the render pass, and HDR tonemapping:
/// scene.render(&mut encoder, &gfx, &swapchain_view, &camera.bind_group.bind_group);
/// ```
pub struct Scene {
    // Shared cube geometry
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    pipeline: Pipeline,

    // Static instances — baked in at construction
    // Kept alive so the GPU buffer remains valid for the culling compute shader bind group.
    #[allow(dead_code)]
    all_instances_buffer: wgpu::Buffer,
    visible_instances_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    total_instance_count: u32,
    cull_pass: CullingPass,
    culling_bind_group: wgpu::BindGroup,

    /// Dynamic instances — rebuilt every frame
    dynamic_scene: DynamicScene,

    /// Optional skybox
    skybox: Option<SkyboxData>,

    /// HDR intermediate texture + tonemap pipeline
    hdr: HdrPipeline,

    /// The camera scene
    pub camera: Camera,
}

impl Scene {
    /// Create a new scene.
    ///
    /// `static_cubes` is the flat list of all cubes for the fixed geometry.
    /// Use [`load_model`](crate::cube::load_model) to produce it.
    ///
    /// `dynamic_max_instances` caps the **total number of cubes** across all
    /// live dynamic models and pre-allocates the dynamic GPU buffer accordingly.
    ///
    /// `peak_brightness_nits` controls HDR tonemapping (ignored in SDR mode).
    pub fn new(
        ctx: &EngineContext,
        camera: Camera,
        static_cubes: &[ModelCube],
        dynamic_max_instances: usize,
    ) -> Self {
        let gfx = &ctx.gfx;
        let device = gfx.device();

        let tonemap_mode = if gfx.is_hdr_active {
            TonemappingMode::Hdr
        } else {
            TonemappingMode::Sdr
        };
        let hdr = HdrPipeline::new(
            device,
            &gfx.surface_config,
            tonemap_mode,
            gfx.peak_brightness_nits,
        );
        let target_format = hdr.format();

        // Shared unit-cube geometry
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Bake static cube positions into GPU instances
        let instance_data: Vec<CubeRaw> = static_cubes
            .iter()
            .map(|c| {
                Cube {
                    model: Matrix4::from_translation(c.position),
                    color: Vector4::new(c.color.x, c.color.y, c.color.z, 1.0),
                }
                .into_raw()
            })
            .collect();
        let total_instance_count = instance_data.len() as u32;

        let static_usage =
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;

        let all_instances_buffer = if instance_data.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Static Instances Buffer"),
                size: 4,
                usage: static_usage,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Static Instances Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: static_usage,
            })
        };

        let visible_size = (instance_data.len() * size_of::<CubeRaw>()).max(4) as u64;
        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instances Buffer"),
            size: visible_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // indirect: [index_count, instance_count, first_index, base_vertex, first_instance]
        let indirect_args = [INDICES.len() as u32, 0u32, 0u32, 0u32, 0u32];
        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Draw Buffer"),
            contents: bytemuck::cast_slice(&indirect_args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let shader = Shader::new(CUBE_SHADER, device, Some("Cube Shader"));
        let pipeline = Pipeline::new_v2(
            device,
            &[&camera.bind_group.layout],
            &[Vertex::desc(), Cube::desc()],
            &shader,
            target_format,
            Some(wgpu::TextureFormat::Depth32Float),
            wgpu::PrimitiveTopology::TriangleList,
            Some("Cubes Pipeline"),
        );

        let cull_pass = CullingPass::new(device, &camera.bind_group.layout);
        let culling_bind_group = cull_pass.create_bind_group(
            device,
            &all_instances_buffer,
            &visible_instances_buffer,
            &indirect_buffer,
        );

        let dynamic_scene = DynamicScene::new(device, dynamic_max_instances.max(1));

        Self {
            vertex_buffer,
            index_buffer,
            pipeline,
            all_instances_buffer,
            visible_instances_buffer,
            indirect_buffer,
            total_instance_count,
            cull_pass,
            culling_bind_group,
            dynamic_scene,
            skybox: None,
            hdr,
            camera,
        }
    }

    // -------------------------------------------------------------------------
    // Skybox
    // -------------------------------------------------------------------------

    /// Load a skybox from raw equirectangular HDR bytes.
    ///
    /// `cubemap_resolution` is the edge length of each cubemap face.
    /// Replaces any previously loaded skybox.
    pub fn set_skybox_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        cubemap_resolution: u32,
    ) -> Result<()> {
        let loader = HdrLoader::new(device);
        let sky_texture = loader.from_equirectangular_bytes(
            device,
            queue,
            bytes,
            cubemap_resolution,
            Some("Skybox Texture"),
        )?;

        let bind_group = BindGroupBuilder::new()
            .add_cube_texture(
                0,
                sky_texture.view().clone(),
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: false },
            )
            .add_sampler(
                1,
                sky_texture.sampler().clone(),
                wgpu::SamplerBindingType::NonFiltering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(device, Some("Skybox Bind Group"));

        let skybox_shader = Shader::new(SKYBOX_SHADER, device, Some("Skybox Shader"));
        let pipeline = Pipeline::new_skybox(
            device,
            &[&self.camera.bind_group.layout, &bind_group.layout],
            &skybox_shader,
            self.hdr.format(),
            wgpu::TextureFormat::Depth32Float,
            Some("Skybox Pipeline"),
        );

        self.skybox = Some(SkyboxData {
            pipeline,
            bind_group,
        });
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Dynamic models
    // -------------------------------------------------------------------------

    /// Add a dynamic model to the scene. Returns a stable ID used to move or remove it.
    pub fn add_dynamic(&mut self, model: DynamicModel) -> usize {
        self.dynamic_scene.add(model)
    }

    /// Remove a dynamic model by ID.
    pub fn remove_dynamic(&mut self, id: usize) {
        self.dynamic_scene.remove(id);
    }

    /// Get a shared reference to a dynamic model by ID.
    pub fn get_dynamic(&self, id: usize) -> Option<&DynamicModel> {
        self.dynamic_scene.get(id)
    }

    /// Get a mutable reference to a dynamic model by ID.
    ///
    /// The updated transform is uploaded to the GPU on the next [`render`] call.
    pub fn get_dynamic_mut(&mut self, id: usize) -> Option<&mut DynamicModel> {
        self.dynamic_scene.get_mut(id)
    }

    // -------------------------------------------------------------------------
    // Stats
    // -------------------------------------------------------------------------

    /// Total number of static cube instances.
    pub fn static_count(&self) -> u32 {
        self.total_instance_count
    }

    /// Total number of live dynamic cube instances across all dynamic models.
    pub fn dynamic_count(&self) -> u32 {
        self.dynamic_scene.live_count()
    }

    // -------------------------------------------------------------------------
    // Frame loop
    // -------------------------------------------------------------------------

    /// Resize the internal HDR texture to match the new window dimensions.
    ///
    /// Call this from [`Game::resize`] whenever the window is resized.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.hdr.resize(device, width, height);
    }

    /// Render the full scene to `swapchain_view`.
    ///
    /// Internally this:
    /// 1. Uploads changed dynamic transforms to the GPU.
    /// 2. Runs the GPU frustum culling compute pass (if enabled).
    /// 3. Opens an HDR render pass and draws static models, dynamic models,
    ///    and the skybox (if set).
    /// 4. Tonemaps the HDR result to `swapchain_view`.
    pub fn render(
        &self,
        ctx: &EngineContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let queue = &ctx.gfx.queue;

        // --- Upload dynamic transforms + GPU culling (mutable, before render pass) ---
        self.dynamic_scene.update_gpu(queue);

        if self.total_instance_count > 0 {
            queue.write_buffer(&self.indirect_buffer, 4, bytemuck::bytes_of(&0u32));
            self.cull_pass.cull(
                encoder,
                &self.camera.bind_group.bind_group,
                &self.culling_bind_group,
                self.total_instance_count,
            );
        }

        // --- Render pass on the HDR intermediate texture ---
        // All borrows inside this block are shared; the mutable work above is done.
        {
            let color_attachment = Gfx::color_attachments_from_view(self.hdr.view());
            let depth_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &ctx.gfx.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            };
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Scene Render Pass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(depth_attachment),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Cube pipeline — shared by static and dynamic
            render_pass.set_pipeline(&self.pipeline.pipeline);
            render_pass.set_bind_group(0, &self.camera.bind_group.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Static geometry — always drawn via indirect after GPU culling
            if self.total_instance_count > 0 {
                render_pass.set_vertex_buffer(1, self.visible_instances_buffer.slice(..));
                render_pass.draw_indexed_indirect(&self.indirect_buffer, 0);
            }

            // Dynamic geometry
            if self.dynamic_scene.live_count() > 0 {
                render_pass.set_vertex_buffer(1, self.dynamic_scene.buffer().slice(..));
                render_pass.draw_indexed(
                    0..INDICES.len() as u32,
                    0,
                    0..self.dynamic_scene.live_count(),
                );
            }

            // Skybox (rendered last, at the far plane)
            if let Some(skybox) = &self.skybox {
                render_pass.set_pipeline(&skybox.pipeline.pipeline);
                render_pass.set_bind_group(0, &self.camera.bind_group.bind_group, &[]);
                render_pass.set_bind_group(1, &skybox.bind_group.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        // --- Tonemap HDR → swapchain ---
        self.hdr.process(encoder, view);
    }
}
