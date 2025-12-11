use cgmath::InnerSpace;
use env_logger;
use etib::BufferExt;
use etib::Game;
use log::info;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

struct MyGame<'vertex> {
    window: Option<Arc<Window>>,
    gfx: Option<etib::Gfx>,
    my_gfx: Option<MyGfx<'vertex>>,
    time: etib::TimeState,
    is_initialized: bool,
    model_path: String,
    camera_controller: etib::CameraController,
    is_isometric: bool,
    cursor_grabbed: bool,
    frame_count: u32,
    fps_update_timer: f32,
}

struct MyGfx<'vertex> {
    camera: etib::Camera,
    pipeline: etib::Pipeline,
    vertex_buffer: etib::VertexBuffer<'vertex, etib::Vertex>,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
}

impl MyGame<'_> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "ETIB".to_owned();
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let window_size = window.inner_size();

        let gfx = etib::Gfx::new(window.clone());
        let device = gfx.device();

        let camera = if self.is_isometric {
            etib::Camera::new(
                &device,
                (50.0, 50.0, 50.0).into(), // Isometric position
                (0.0, 0.0, 0.0).into(),    // Looking at the origin
                cgmath::Vector3::unit_y(),
                window_size.width as f32 / window_size.height as f32,
                etib::Projection::Orthographic { scale: 50.0 },
                -200.0, // adjusted near/far for ortho
                200.0,
            )
        } else {
            etib::Camera::new(
                &device,
                (0.0, 30.0, 80.0).into(),  // Zoomed out and higher up
                (0.0, 10.0, 0.0).into(),    // Looking towards center
                cgmath::Vector3::unit_y(),
                window_size.width as f32 / window_size.height as f32,
                etib::Projection::Perspective { fovy: 45.0 },
                0.1,
                500.0,  // Increased far plane for larger scene
            )
        };

        // Load cube positions from model file
        let model_cubes = etib::load_model(&self.model_path).expect("Failed to load model file");

        // Create instance data for all cubes
        let instance_data: Vec<etib::CubeRaw> = model_cubes
            .iter()
            .map(|cube| {
                let cube_instance = etib::Cube {
                    model: cgmath::Matrix4::<f32>::from_translation(cube.position).into(),
                    color: cgmath::Vector4::from([cube.color.x, cube.color.y, cube.color.z, 1.0]),
                };
                cube_instance.into_raw()
            })
            .collect();

        let instance_count = instance_data.len() as u32;

        // Create instance buffer
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let shader_str = include_str!("../src/shaders/shader.wgsl");
        let shader = etib::Shader::new(shader_str, &device, None);
        let vertex_buffer = device.create_vertex_buffer(etib::VERTICES, etib::Vertex::desc());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(etib::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline with both vertex and instance buffer layouts
        let pipeline = etib::Pipeline::new_with_layouts(
            &device,
            &[&camera.uniform.layout],  // Only camera uniform now
            &shader,
            &gfx.surface_config,
            &[
                etib::Vertex::desc(),  // Vertex buffer layout
                etib::Cube::desc(),    // Instance buffer layout
            ],
        );

        let my_gfx = MyGfx {
            camera,
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_count,
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
            let cube_count = my_gfx.instance_count;
            if let Some(window) = &self.window {
                window.set_title(&format!("ETIB - {:.0} FPS - {} cubes", fps, cube_count));
            }
            self.frame_count = 0;
            self.fps_update_timer = 0.0;
        }

        // Update camera based on controller input
        self.camera_controller.update_camera(&mut my_gfx.camera, self.time.dt);
        let new_matrix = my_gfx.camera.update_matrix();
        gfx.queue.write_buffer(
            &my_gfx.camera.buffer,
            0,
            bytemuck::cast_slice(&[Into::<[[f32; 4]; 4]>::into(new_matrix)]),
        );

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let color_attachments = [Some(etib::Gfx::color_attachments_from_view(&view))];
            let mut render_pass = encoder.begin_render_pass(&gfx.render_pass(&color_attachments));

            render_pass.set_pipeline(&my_gfx.pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.uniform.bind_group, &[]);
            render_pass.set_vertex_buffer(0, my_gfx.vertex_buffer.buffer.slice(..));
            render_pass.set_vertex_buffer(1, my_gfx.instance_buffer.slice(..));
            render_pass.set_index_buffer(my_gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw all cubes with a single instanced draw call!
            render_pass.draw_indexed(
                0..etib::INDICES.len() as u32,
                0,
                0..my_gfx.instance_count,
            );
        }

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl etib::Game for MyGame<'_> {
    fn gfx(&mut self) -> &mut etib::Gfx {
        self.gfx.as_mut().unwrap()
    }

    fn time_state(&self) -> &etib::TimeState {
        &self.time
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized
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

                // Press ESC to release cursor, press C to grab cursor
                if event.state == winit::event::ElementState::Pressed {
                    match event.physical_key {
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
                                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                    .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Locked));
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
                        let new_matrix = my_gfx.camera.update_matrix();
                        gfx.queue.write_buffer(
                            &my_gfx.camera.buffer,
                            0,
                            bytemuck::cast_slice(&[Into::<[[f32; 4]; 4]>::into(new_matrix)]),
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
        if self.cursor_grabbed {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Initializing ETIB");

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <model_file> [--isometric]", args[0]);
        eprintln!("Example: {} examples/models/cat.model", args[0]);
        std::process::exit(1);
    }
    let model_path = args[1].clone();
    let is_isometric = args.contains(&"--isometric".to_string());

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
    
    let mut camera_controller = etib::CameraController::new(20.0, 0.003);  // Increased speed for larger scene
    if is_isometric {
        camera_controller.mode = etib::CameraMode::Isometric;
    }
    camera_controller.yaw = initial_yaw;
    camera_controller.pitch = initial_pitch;
    
    let mut game = MyGame {
        window: None,
        gfx: None,
        my_gfx: None,
        time: etib::TimeState::default(),
        is_initialized: false,
        model_path,
        camera_controller,
        is_isometric,
        cursor_grabbed: false,
        frame_count: 0,
        fps_update_timer: 0.0,
    };
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    etib::run(&mut game, event_loop)?;

    Ok(())
}
