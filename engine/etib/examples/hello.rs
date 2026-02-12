use std::sync::Arc;

use cgmath::InnerSpace;
use log::info;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

use etib::Game;
use etib_core::bindgroup::BindGroupBuilder;
use etib_core::buffer::BufferExt;

struct MyGame<'vertex> {
    window: Option<Arc<Window>>,
    gfx: Option<etib::Gfx>,
    my_gfx: Option<MyGfx<'vertex>>,
    time: etib::time::TimeState,
    is_initialized: bool,
    model_path: String,
    config_path: Option<String>,
    camera_controller: etib::camera::CameraController,
    is_isometric: bool,
    cursor_grabbed: bool,
    frame_count: u32,
    fps_update_timer: f32,
    enable_culling: bool, // Toggle frustum culling with F1
}

struct MyGfx<'vertex> {
    camera: etib::camera::Camera,
    pipeline: etib_core::pipeline::Pipeline,
    hdr: etib::hdr::HdrPipeline,
    skybox_pipeline: etib_core::pipeline::Pipeline,
    skybox: etib_core::bindgroup::BindGroup,
    vertex_buffer: etib_core::buffer::VertexBuffer<'vertex, etib::Vertex>,
    index_buffer: wgpu::Buffer,

    // GPU Culling resources
    cull_pass: etib::cube::CullingPass,
    culling_bind_group: wgpu::BindGroup,
    all_instances_buffer: wgpu::Buffer, // Contains all instances (input to cull shader)
    visible_instances_buffer: wgpu::Buffer, // Contains visible instances (output from cull shader)
    indirect_buffer: wgpu::Buffer,      // Contains draw calls parameters

    // Occlusion Culling resources
    hiz_buffer: etib::cube::HiZBuffer,
    occlusion_pass: etib::cube::OcclusionPass,
    last_frame_depth: wgpu::Texture,
    last_frame_depth_view: wgpu::TextureView,

    total_instance_count: u32,
    last_camera_eye: cgmath::Point3<f32>,
    last_camera_target: cgmath::Point3<f32>,
}

impl MyGame<'_> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "ETIB".to_owned();
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let window_size = window.inner_size();

        let gfx = etib::Gfx::new(window.clone(), self.config_path.as_deref());
        let device = gfx.device();

        let hdr_loader = etib::hdr::HdrLoader::new(&device);
        let sky_bytes = include_bytes!("../examples/sky.hdr");
        let sky_texture = hdr_loader
            .from_equirectangular_bytes(&device, &gfx.queue, sky_bytes, 1080, Some("Sky Texture"))
            .unwrap();

        let camera = if self.is_isometric {
            etib::camera::Camera::new(
                &device,
                (50.0, 50.0, 50.0).into(), // Isometric position
                (0.0, 0.0, 0.0).into(),    // Looking at the origin
                cgmath::Vector3::unit_y(),
                window_size.width as f32 / window_size.height as f32,
                etib::camera::Projection::Orthographic { scale: 50.0 },
                -200.0, // adjusted near/far for ortho
                200.0,
            )
        } else {
            etib::camera::Camera::new(
                &device,
                (0.0, 30.0, 80.0).into(), // Zoomed out and higher up
                (0.0, 10.0, 0.0).into(),  // Looking towards center
                cgmath::Vector3::unit_y(),
                window_size.width as f32 / window_size.height as f32,
                etib::camera::Projection::Perspective { fovy: 45.0 },
                0.1,
                500.0, // Increased far plane for larger scene
            )
        };

        // Load cube positions from model file
        let model_cubes =
            etib::cube::load_model(&self.model_path).expect("Failed to load model file");

        // Create instance data for all cubes
        let instance_data: Vec<etib::cube::CubeRaw> = model_cubes
            .iter()
            .map(|cube| {
                let cube_instance = etib::cube::Cube {
                    model: cgmath::Matrix4::<f32>::from_translation(cube.position),
                    color: cgmath::Vector4::from([cube.color.x, cube.color.y, cube.color.z, 1.0]),
                };
                cube_instance.into_raw()
            })
            .collect();

        let instance_count = instance_data.len() as u32;

        // Create buffer with all instances (Storage for compute, Vertex for direct rendering when culling disabled)
        let all_instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("All Instances Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        // Create buffer for visible instances (Storage for compute output, Vertex for rendering)
        let visible_instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instances Buffer"),
            size: (instance_data.len() * std::mem::size_of::<etib::cube::CubeRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // Create indirect buffer for draw calls
        // [index_count, instance_count, first_index, base_vertex, first_instance]
        let indirect_args = [etib::cube::INDICES.len() as u32, 0, 0, 0, 0];
        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Draw Buffer"),
            contents: bytemuck::cast_slice(&indirect_args),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let shader_str = include_str!("../src/shaders/shader.wgsl");
        let shader = etib_core::shader::Shader::new(shader_str, &device, None);
        let vertex_buffer = device.create_vertex_buffer(etib::cube::VERTICES, etib::Vertex::desc());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(etib::cube::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline with both vertex and instance buffer layouts
        let pipeline = etib_core::pipeline::Pipeline::new_v2(
            &device,
            &[&camera.bind_group.layout], // Only camera uniform now
            &[
                etib::Vertex::desc(),     // Vertex buffer layout
                etib::cube::Cube::desc(), // Instance buffer layout
            ],
            &shader,
            wgpu::TextureFormat::Rgba16Float, // For HDR support
            Some(wgpu::TextureFormat::Depth32Float),
            wgpu::PrimitiveTopology::TriangleList,
            Some("Cubes Pipeline"),
        );

        // Configure HDR pipeline based on swapchain format
        let config =
            etib::config::EngineConfig::load_from_file(self.config_path.as_deref().unwrap_or("config.json"));
        let tonemap_mode = if gfx.is_hdr_active {
            etib::hdr::TonemappingMode::Hdr
        } else {
            etib::hdr::TonemappingMode::Sdr
        };
        let hdr = etib::hdr::HdrPipeline::new(
            &device,
            &gfx.surface_config,
            tonemap_mode,
            config.peak_brightness_nits,
        );

        // Skybox pipeline
        let skybox_shader_str = include_str!("../src/shaders/skybox.wgsl");
        let skybox_shader =
            etib_core::shader::Shader::new(skybox_shader_str, &device, Some("Skybox shader"));
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
            &device,
            &[&camera.bind_group.layout, &skybox.layout],
            &skybox_shader,
            hdr.format(),
            wgpu::TextureFormat::Depth32Float,
            Some("Skybox pipeline"),
        );

        // Store initial camera position for movement detection
        let initial_eye = camera.eye;
        let initial_target = camera.target;

        // Initialize GPU Culling Pass
        let cull_pass = etib::cube::CullingPass::new(&device, &camera.bind_group.layout);

        // Initialize Hi-Z Buffer and Occlusion Pass
        let hiz_buffer = etib::cube::HiZBuffer::new(
            &device,
            window_size.width.next_power_of_two(),
            window_size.height.next_power_of_two(),
        );
        let occlusion_pass = etib::cube::OcclusionPass::new(&device);

        // Create texture to store previous frame's depth
        let last_frame_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Last Frame Depth"),
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let last_frame_depth_view =
            last_frame_depth.create_view(&wgpu::TextureViewDescriptor::default());

        // Clear last frame depth to 1.0 (far) to avoid culling everything on first frame
        // Actually, we can't easily clear without a render pass or write_texture.
        // Let's assume it's okay for one frame, or we can use encoder.clear_texture if available (not in standard wgpu?)
        // Standard way: RenderPass with LoadOp::Clear. But we need a depth view.
        // Just create a temporary command encoder and clear it.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Init Clear"),
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Last Depth"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &last_frame_depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        gfx.queue.submit(Some(encoder.finish()));

        let culling_bind_group = cull_pass.create_bind_group(
            &device,
            &all_instances_buffer,
            &visible_instances_buffer,
            &indirect_buffer,
            &hiz_buffer.full_view,
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
            hiz_buffer,
            occlusion_pass,
            last_frame_depth,
            last_frame_depth_view,
            total_instance_count: instance_count,
            last_camera_eye: initial_eye,
            last_camera_target: initial_target,
        };

        self.window = Some(window);
        self.gfx = Some(gfx);
        self.my_gfx = Some(my_gfx);
        self.is_initialized = true;
    }

    fn render(&mut self) {
        let gfx = self.gfx.as_ref().unwrap();
        let my_gfx = self.my_gfx.as_mut().unwrap();
        let (frame, view) = gfx.get_next_frame();

        // Update FPS counter
        self.frame_count += 1;
        self.fps_update_timer += self.time.dt;

        // Update title bar every 0.5 seconds
        if self.fps_update_timer >= 0.5 {
            let fps = self.frame_count as f32 / self.fps_update_timer;
            let total_count = my_gfx.total_instance_count;
            let visible_info = if self.enable_culling { "GPU" } else { "All" };
            let culling_status = if self.enable_culling {
                "ON (F1 to disable)"
            } else {
                "OFF (F1 to enable)"
            };
            if let Some(window) = &self.window {
                window.set_title(&format!(
                    "ETIB - {:.0} FPS - Visible: {} / Total: {} - Culling: {}",
                    fps, visible_info, total_count, culling_status
                ));
            }
            self.frame_count = 0;
            self.fps_update_timer = 0.0;
        }

        // Update camera based on controller input
        self.camera_controller
            .update_camera(&gfx.queue, &mut my_gfx.camera, self.time.dt);

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // GPU Culling
        if self.enable_culling {
            // 1. Generate Hi-Z Buffer from last frame's depth
            /*
            my_gfx.occlusion_pass.generate(
                &gfx.device,
                &mut encoder,
                &my_gfx.hiz_buffer,
                &my_gfx.last_frame_depth_view,
            );
            */

            // 2. Reset instance count in indirect buffer to 0
            // Offset 4 is where instance_count is located in DrawIndexedIndirectArgs
            gfx.queue
                .write_buffer(&my_gfx.indirect_buffer, 4, &[0, 0, 0, 0]);

            // 3. Dispatch Culling Compute Shader
            my_gfx.cull_pass.cull(
                &mut encoder,
                &my_gfx.camera.bind_group.bind_group,
                &my_gfx.culling_bind_group,
                my_gfx.total_instance_count,
            );
        }

        {
            // Render first on HDR texture
            let color_attachments = [Some(etib::Gfx::color_attachments_from_view(
                &my_gfx.hdr.view(),
            ))];
            let mut render_pass = encoder.begin_render_pass(&gfx.render_pass(&color_attachments));

            // Render Scene
            render_pass.set_pipeline(&my_gfx.pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.bind_group.bind_group, &[]);
            render_pass.set_vertex_buffer(0, my_gfx.vertex_buffer.buffer.slice(..));
            render_pass.set_index_buffer(my_gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            if self.enable_culling {
                // Use visible instances buffer and indirect draw
                render_pass.set_vertex_buffer(1, my_gfx.visible_instances_buffer.slice(..));
                render_pass.draw_indexed_indirect(&my_gfx.indirect_buffer, 0);
            } else {
                // Use all instances buffer and direct draw
                render_pass.set_vertex_buffer(1, my_gfx.all_instances_buffer.slice(..));
                render_pass.draw_indexed(
                    0..etib::cube::INDICES.len() as u32,
                    0,
                    0..my_gfx.total_instance_count,
                );
            }

            render_pass.set_pipeline(&my_gfx.skybox_pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.bind_group.bind_group, &[]);
            render_pass.set_bind_group(1, &my_gfx.skybox.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Copy current depth to last_frame_depth for next frame
        /*
        if self.enable_culling {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &gfx.depth_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::DepthOnly,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &my_gfx.last_frame_depth,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: gfx.surface_config.width,
                    height: gfx.surface_config.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        */

        // Tonemap the HDR Rgba16Float Texture to the original Rgba8UnormSRGB Texture
        my_gfx.hdr.process(&mut encoder, &view);

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl Game for MyGame<'_> {
    fn gfx(&mut self) -> &mut etib::Gfx {
        self.gfx.as_mut().unwrap()
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    fn time_state(&self) -> &etib::time::TimeState {
        &self.time
    }
}

impl ApplicationHandler for MyGame<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Handle camera movement keys
                self.camera_controller.process_keyboard(event.clone());

                // Press ESC to release cursor, press C to grab cursor, press F1 to toggle frustum culling
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
                            if let Some(window) = &self.window {
                                self.cursor_grabbed = false;
                                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
                                window.set_cursor_visible(true);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyC) => {
                            if let Some(window) = &self.window {
                                self.cursor_grabbed = true;
                                let _ = window
                                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                    .or_else(|_| {
                                        window
                                            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                    });
                                window.set_cursor_visible(false);
                            }
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(&delta);
            }
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.reconfigure_surface_size(size);

                    // Update camera aspect ratio
                    if let Some(my_gfx) = &mut self.my_gfx {
                        my_gfx.camera.aspect = size.width as f32 / size.height as f32;
                        my_gfx.camera.update_matrix(&gfx.queue);

                        my_gfx.hdr.resize(gfx.device(), size.width, size.height);

                        // Recreate Hi-Z and Last Frame Depth
                        my_gfx.hiz_buffer = etib::cube::HiZBuffer::new(
                            gfx.device(),
                            size.width.next_power_of_two(),
                            size.height.next_power_of_two(),
                        );

                        my_gfx.last_frame_depth =
                            gfx.device().create_texture(&wgpu::TextureDescriptor {
                                label: Some("Last Frame Depth"),
                                size: wgpu::Extent3d {
                                    width: size.width,
                                    height: size.height,
                                    depth_or_array_layers: 1,
                                },
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: wgpu::TextureDimension::D2,
                                format: wgpu::TextureFormat::Depth32Float,
                                usage: wgpu::TextureUsages::TEXTURE_BINDING
                                    | wgpu::TextureUsages::COPY_DST
                                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                                view_formats: &[],
                            });
                        my_gfx.last_frame_depth_view = my_gfx
                            .last_frame_depth
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        // Recreate culling bind group with new Hi-Z view
                        my_gfx.culling_bind_group = my_gfx.cull_pass.create_bind_group(
                            gfx.device(),
                            &my_gfx.all_instances_buffer,
                            &my_gfx.visible_instances_buffer,
                            &my_gfx.indirect_buffer,
                            &my_gfx.hiz_buffer.full_view,
                        );
                    }

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.is_initialized() {
                    self.render();
                }

                self.time.tick();
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Only process mouse movement when cursor is grabbed
        if self.cursor_grabbed
            && let DeviceEvent::MouseMotion { delta } = event
        {
            self.camera_controller.process_mouse(delta.0, delta.1);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Initializing ETIB");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <model_file> [--isometric] [--config <path>]", args[0]);
        eprintln!("Example: {} examples/models/cat.model", args[0]);
        eprintln!("Example: {} examples/models/cat.model --config my_config.json", args[0]);
        std::process::exit(1);
    }
    let model_path = args[1].clone();
    let is_isometric = args.contains(&"--isometric".to_string());

    // Parse --config argument
    let config_path = args
        .iter()
        .position(|arg| arg == "--config")
        .and_then(|i| args.get(i + 1).cloned());

    // Check if the model file exists
    if !std::path::Path::new(&model_path).exists() {
        eprintln!("Error: Model file '{}' does not exist", model_path);
        std::process::exit(1);
    }

    // Calculate initial camera angles for FPS camera
    // Camera starts at (0, 30, 80) looking at (0, 10, 0) - zoomed out view
    let initial_eye = cgmath::Point3::new(0.0_f32, 30.0, 80.0);
    let initial_target = cgmath::Point3::new(0.0_f32, 10.0, 0.0);
    let forward = (initial_target - initial_eye).normalize();
    let initial_yaw: f32 = forward.z.atan2(forward.x);
    let initial_pitch: f32 = forward.y.asin();

    let mut camera_controller = etib::camera::CameraController::new(20.0, 0.003); // Increased speed for larger scene
    if is_isometric {
        camera_controller.mode = etib::camera::CameraMode::Isometric;
    }
    camera_controller.yaw = initial_yaw;
    camera_controller.pitch = initial_pitch;

    let config =
        etib::config::EngineConfig::load_from_file(config_path.as_deref().unwrap_or("config.json"));

    let mut game = MyGame {
        window: None,
        gfx: None,
        my_gfx: None,
        time: etib::time::TimeState::new(),
        is_initialized: false,
        model_path,
        config_path,
        camera_controller,
        is_isometric,
        cursor_grabbed: false,
        frame_count: 0,
        fps_update_timer: 0.0,
        enable_culling: config.culling,
    };
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    etib::run(&mut game, event_loop)?;

    Ok(())
}
