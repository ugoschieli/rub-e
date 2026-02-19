use cgmath::{InnerSpace, Rotation3};
use clap::Parser;
use log::info;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::config::EngineConfig;
use etib::{EngineContext, Game};
use etib_core::bindgroup::BindGroupBuilder;
use etib_core::buffer::BufferExt;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Engine configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[arg(long)]
    isometric: bool,

    /// Model file to render.
    /// Loaded once as static geometry and once as a dynamic instance that orbits.
    model_path: String,
}

struct MyParams {
    model_path: String,
    is_isometric: bool,
}

struct MyGame<'vertex> {
    my_gfx: Option<MyGfx<'vertex>>,
    camera_controller: etib::camera::CameraController,
    cursor_grabbed: bool,
    frame_count: u32,
    fps_update_timer: f32,
    enable_culling: bool,
    /// Accumulated time used to animate dynamic models.
    time_elapsed: f32,
    /// ID of the orbiting dynamic model returned by [`DynamicScene::add`].
    orbiting_id: usize,
}

struct MyGfx<'vertex> {
    camera: etib::camera::Camera,
    pipeline: etib_core::pipeline::Pipeline,
    hdr: etib::hdr::HdrPipeline,
    skybox_pipeline: etib_core::pipeline::Pipeline,
    skybox: etib_core::bindgroup::BindGroup,
    vertex_buffer: etib_core::buffer::VertexBuffer<'vertex, etib::Vertex>,
    index_buffer: wgpu::Buffer,

    // Static scene: loaded once at init, never updated.
    cull_pass: etib::cube::CullingPass,
    culling_bind_group: wgpu::BindGroup,
    all_instances_buffer: wgpu::Buffer,
    visible_instances_buffer: wgpu::Buffer,
    indirect_buffer: wgpu::Buffer,
    total_instance_count: u32,

    // Dynamic scene: models loaded from files, moved independently at runtime.
    dynamic_scene: etib::cube::DynamicScene,
}

impl Game for MyGame<'_> {
    type InitParams = MyParams;

    fn init(ctx: &mut EngineContext, params: Self::InitParams) -> Self {
        let initial_eye = cgmath::Point3::new(0.0_f32, 30.0, 80.0);
        let initial_target = cgmath::Point3::new(0.0_f32, 10.0, 0.0);
        let forward = (initial_target - initial_eye).normalize();
        let initial_yaw: f32 = forward.z.atan2(forward.x);
        let initial_pitch: f32 = forward.y.asin();

        let mut camera_controller = etib::camera::CameraController::new(20.0, 0.003);
        if params.is_isometric {
            camera_controller.mode = etib::camera::CameraMode::Isometric;
        }
        camera_controller.yaw = initial_yaw;
        camera_controller.pitch = initial_pitch;

        let gfx = &ctx.gfx;
        let device = gfx.device();

        let hdr_loader = etib::hdr::HdrLoader::new(device);
        let sky_bytes = include_bytes!("../examples/sky.hdr");
        let sky_texture = hdr_loader
            .from_equirectangular_bytes(device, &gfx.queue, sky_bytes, 1080, Some("Sky Texture"))
            .unwrap();

        let camera = if params.is_isometric {
            etib::camera::Camera::new(
                device,
                (50.0, 50.0, 50.0).into(),
                (0.0, 0.0, 0.0).into(),
                cgmath::Vector3::unit_y(),
                ctx.window_size().width as f32 / ctx.window_size().height as f32,
                etib::camera::Projection::Orthographic { scale: 50.0 },
                -200.0,
                200.0,
            )
        } else {
            etib::camera::Camera::new(
                device,
                (0.0, 30.0, 80.0).into(),
                (0.0, 10.0, 0.0).into(),
                cgmath::Vector3::unit_y(),
                ctx.window_size().width as f32 / ctx.window_size().height as f32,
                etib::camera::Projection::Perspective { fovy: 45.0 },
                0.1,
                500.0,
            )
        };

        // --- Static scene ---
        // Load the model once; its cube positions are baked into the instance buffer
        // and never touched again.
        let model_cubes =
            etib::cube::load_model(&params.model_path).expect("Failed to load model file");

        let instance_data: Vec<etib::cube::CubeRaw> = model_cubes
            .iter()
            .map(|cube| {
                etib::cube::Cube {
                    model: cgmath::Matrix4::<f32>::from_translation(cube.position),
                    color: cgmath::Vector4::from([cube.color.x, cube.color.y, cube.color.z, 1.0]),
                }
                .into_raw()
            })
            .collect();

        let instance_count = instance_data.len() as u32;

        let all_instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Static Instances Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instances Buffer"),
            size: (instance_data.len() * size_of::<etib::cube::CubeRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let indirect_args = [etib::cube::INDICES.len() as u32, 0, 0, 0, 0];
        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Draw Buffer"),
            contents: bytemuck::cast_slice(&indirect_args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        // --- Dynamic scene ---
        // The same model file is loaded again as a DynamicModel. Its group transform
        // (position/rotation/scale) is updated every frame so it orbits the static version.
        //
        // `max_instances` is set to the cube count of one model copy. Add more if you
        // want multiple dynamic instances of varying models loaded at runtime.
        let mut orbiting = etib::cube::DynamicModel::load("models/cat.model")
            .expect("Failed to load dynamic model");
        orbiting.position = cgmath::Vector3::new(30.0, 0.0, 0.0);

        let mut dynamic_scene = etib::cube::DynamicScene::new(device, orbiting.cube_count().max(1));
        let orbiting_id = dynamic_scene.add(orbiting);

        // Upload initial positions before the first frame.
        dynamic_scene.update_gpu(&gfx.queue);

        // --- Shared render pipeline (static and dynamic use the same shader/layout) ---
        let shader_str = include_str!("../src/shaders/shader.wgsl");
        let shader = etib_core::shader::Shader::new(shader_str, device, None);
        let vertex_buffer = device.create_vertex_buffer(etib::cube::VERTICES, etib::Vertex::desc());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(etib::cube::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline = etib_core::pipeline::Pipeline::new_v2(
            device,
            &[&camera.bind_group.layout],
            &[etib::Vertex::desc(), etib::cube::Cube::desc()],
            &shader,
            wgpu::TextureFormat::Rgba16Float,
            Some(wgpu::TextureFormat::Depth32Float),
            wgpu::PrimitiveTopology::TriangleList,
            Some("Cubes Pipeline"),
        );

        let tonemap_mode = if gfx.is_hdr_active {
            etib::hdr::TonemappingMode::Hdr
        } else {
            etib::hdr::TonemappingMode::Sdr
        };
        let hdr = etib::hdr::HdrPipeline::new(
            device,
            &gfx.surface_config,
            tonemap_mode,
            ctx.config.peak_brightness_nits,
        );

        let skybox_shader_str = include_str!("../src/shaders/skybox.wgsl");
        let skybox_shader =
            etib_core::shader::Shader::new(skybox_shader_str, device, Some("Skybox shader"));
        let skybox = BindGroupBuilder::new()
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
            .build(device, Some("Skybox bind group"));
        let skybox_pipeline = etib_core::pipeline::Pipeline::new_skybox(
            device,
            &[&camera.bind_group.layout, &skybox.layout],
            &skybox_shader,
            hdr.format(),
            wgpu::TextureFormat::Depth32Float,
            Some("Skybox pipeline"),
        );

        let cull_pass = etib::cube::CullingPass::new(device, &camera.bind_group.layout);
        let culling_bind_group = cull_pass.create_bind_group(
            device,
            &all_instances_buffer,
            &visible_instances_buffer,
            &indirect_buffer,
        );

        let my_gfx = MyGfx {
            camera,
            pipeline,
            hdr,
            skybox_pipeline,
            skybox,
            vertex_buffer,
            index_buffer,
            cull_pass,
            culling_bind_group,
            all_instances_buffer,
            visible_instances_buffer,
            indirect_buffer,
            total_instance_count: instance_count,
            dynamic_scene,
        };

        MyGame {
            my_gfx: Some(my_gfx),
            camera_controller,
            cursor_grabbed: false,
            frame_count: 0,
            fps_update_timer: 0.0,
            enable_culling: ctx.config.culling,
            time_elapsed: 0.0,
            orbiting_id,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        self.time_elapsed += ctx.time.dt;
        let t = self.time_elapsed;

        let my_gfx = self.my_gfx.as_mut().unwrap();

        // Orbit the dynamic model around the static one, spinning as it goes.
        if let Some(m) = my_gfx.dynamic_scene.get_mut(self.orbiting_id) {
            m.position.x = (t * 0.5).sin() * 30.0;
            m.position.z = (t * 0.5).cos() * 30.0;
            m.rotation = cgmath::Quaternion::from_angle_y(cgmath::Rad(-t * 0.5));
        }

        // Push changed group transforms to the GPU (no-op if nothing changed).
        my_gfx.dynamic_scene.update_gpu(&ctx.gfx.queue);
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        let gfx = &ctx.gfx;
        let my_gfx = self.my_gfx.as_mut().unwrap();
        let (frame, view) = gfx.get_next_frame();

        self.frame_count += 1;
        self.fps_update_timer += ctx.time.dt;

        if self.fps_update_timer >= 0.5 {
            let fps = self.frame_count as f32 / self.fps_update_timer;
            let total_count = my_gfx.total_instance_count;
            let culling_status = if self.enable_culling {
                "ON (F1 to disable)"
            } else {
                "OFF (F1 to enable)"
            };
            ctx.set_window_title(&format!(
                "ETIB - {:.0} FPS  |  Static: {} cubes  |  Dynamic: {} cubes  |  Culling: {}",
                fps,
                total_count,
                my_gfx.dynamic_scene.live_count(),
                culling_status,
            ));
            self.frame_count = 0;
            self.fps_update_timer = 0.0;
        }

        self.camera_controller
            .update_camera(&gfx.queue, &mut my_gfx.camera, ctx.time.dt);

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ETIB command encoder"),
            });

        // GPU frustum culling for the static scene only.
        if self.enable_culling {
            gfx.queue
                .write_buffer(&my_gfx.indirect_buffer, 4, &[0, 0, 0, 0]);

            my_gfx.cull_pass.cull(
                &mut encoder,
                &my_gfx.camera.bind_group.bind_group,
                &my_gfx.culling_bind_group,
                my_gfx.total_instance_count,
            );
        }

        {
            let color_attachments = [Some(etib::Gfx::color_attachments_from_view(
                &my_gfx.hdr.view(),
            ))];
            let mut render_pass = encoder
                .begin_render_pass(&gfx.render_pass(&color_attachments, "ETIB: render pass"));

            render_pass.set_pipeline(&my_gfx.pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.bind_group.bind_group, &[]);
            render_pass.set_vertex_buffer(0, my_gfx.vertex_buffer.buffer.slice(..));
            render_pass.set_index_buffer(my_gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw static instances (optionally culled).
            if self.enable_culling {
                render_pass.set_vertex_buffer(1, my_gfx.visible_instances_buffer.slice(..));
                render_pass.draw_indexed_indirect(&my_gfx.indirect_buffer, 0);
            } else {
                render_pass.set_vertex_buffer(1, my_gfx.all_instances_buffer.slice(..));
                render_pass.draw_indexed(
                    0..etib::cube::INDICES.len() as u32,
                    0,
                    0..my_gfx.total_instance_count,
                );
            }

            // Draw dynamic instances — always direct, no GPU culling needed for small counts.
            if my_gfx.dynamic_scene.live_count() > 0 {
                render_pass.set_vertex_buffer(1, my_gfx.dynamic_scene.buffer().slice(..));
                render_pass.draw_indexed(
                    0..etib::cube::INDICES.len() as u32,
                    0,
                    0..my_gfx.dynamic_scene.live_count(),
                );
            }

            render_pass.set_pipeline(&my_gfx.skybox_pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.bind_group.bind_group, &[]);
            render_pass.set_bind_group(1, &my_gfx.skybox.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        my_gfx.hdr.process(&mut encoder, &view);

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, ctx: &mut EngineContext, size: PhysicalSize<u32>) {
        ctx.gfx.reconfigure_surface_size(size);

        let my_gfx = self.my_gfx.as_mut().unwrap();
        my_gfx.camera.aspect = size.width as f32 / size.height as f32;
        my_gfx.camera.update_matrix(&ctx.gfx.queue);

        my_gfx.hdr.resize(ctx.gfx.device(), size.width, size.height);

        my_gfx.culling_bind_group = my_gfx.cull_pass.create_bind_group(
            ctx.gfx.device(),
            &my_gfx.all_instances_buffer,
            &my_gfx.visible_instances_buffer,
            &my_gfx.indirect_buffer,
        );
    }

    fn device_input(&mut self, _ctx: &mut EngineContext, event: &DeviceEvent) {
        if self.cursor_grabbed {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
        }
    }

    fn input(&mut self, ctx: &mut EngineContext, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.camera_controller.process_keyboard(event.clone());

                if event.state == winit::event::ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::F1) => {
                            self.enable_culling = !self.enable_culling;
                            println!(
                                "Frustum culling: {}",
                                if self.enable_culling { "ON" } else { "OFF" }
                            );
                        }
                        PhysicalKey::Code(KeyCode::Escape) => {
                            self.cursor_grabbed = false;
                            let _ = ctx.set_cursor_grab(winit::window::CursorGrabMode::None);
                            ctx.set_cursor_visible(true);
                        }
                        PhysicalKey::Code(KeyCode::KeyC) => {
                            self.cursor_grabbed = true;
                            let _ = ctx
                                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                .or_else(|_| {
                                    ctx.set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                });
                            ctx.set_cursor_visible(false);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Initializing ETIB");

    let config = EngineConfig::load_from_file(args.config.as_deref().unwrap_or("config.json"));

    let params = MyParams {
        is_isometric: args.isometric,
        model_path: args.model_path,
    };

    etib::run::<MyGame>(config, Some(params))?;

    Ok(())
}
