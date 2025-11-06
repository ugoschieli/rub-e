use env_logger;
use etib::BufferExt;
use etib::Game;
use log::info;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

#[derive(Default)]
struct MyGame<'vertex> {
    window: Option<Arc<Window>>,
    gfx: Option<etib::Gfx>,
    my_gfx: Option<MyGfx<'vertex>>,
    time: etib::TimeState,
    is_initialized: bool,
}

struct MyGfx<'vertex> {
    camera: etib::Camera,
    pipeline: etib::Pipeline,
    vertex_buffer: etib::VertexBuffer<'vertex, etib::Vertex>,
    index_buffer: wgpu::Buffer,
    cube: etib::Uniform,
    cube2: etib::Uniform,
}

impl MyGame<'_> {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "ETIB".to_owned();
        let window = Arc::new(event_loop.create_window(attributes).unwrap());
        let window_size = window.inner_size();

        let gfx = etib::Gfx::new(window.clone());
        let device = gfx.device();

        let camera = etib::Camera::new(
            &device,
            (2.0, 2.0, 3.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            window_size.width as f32 / window_size.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let cube = etib::Uniform::new_with_buffer(
            device,
            &Into::<[[f32; 4]; 4]>::into(cgmath::Matrix4::<f32>::from_translation(
                cgmath::Vector3 {
                    x: 1.,
                    y: 0.,
                    z: 0.,
                },
            )),
        );

        let cube2 = etib::Uniform::new_with_buffer(
            device,
            &Into::<[[f32; 4]; 4]>::into(cgmath::Matrix4::<f32>::from_translation(
                cgmath::Vector3 {
                    x: -1.,
                    y: 0.,
                    z: 0.,
                },
            )),
        );

        let instance_buffer = device.create_vertex_buffer(
            &[Into::<etib::CubeRaw>::into(etib::Cube::new())],
            etib::Cube::desc(),
        );

        let shader_str = include_str!("../src/shaders/shader.wgsl");
        let shader = etib::Shader::new(shader_str, &device, None);
        let vertex_buffer = device.create_vertex_buffer(etib::VERTICES, etib::Vertex::desc());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(etib::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline = etib::Pipeline::new(
            &device,
            &[&camera.uniform.layout, &cube.layout],
            &shader,
            &gfx.surface_config,
            &vertex_buffer,
        );

        let my_gfx = MyGfx {
            camera,
            pipeline,
            vertex_buffer,
            index_buffer,
            cube,
            cube2,
        };

        self.window = Some(window);
        self.gfx = Some(gfx);
        self.my_gfx = Some(my_gfx);
        self.is_initialized = true;
    }

    fn render(&self) {
        let gfx = self.gfx.as_ref().unwrap();
        let my_gfx = self.my_gfx.as_ref().unwrap();
        let (frame, view) = gfx.get_next_frame();

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let color_attachments = [Some(etib::Gfx::color_attachments_from_view(&view))];
            let mut render_pass = encoder.begin_render_pass(&gfx.render_pass(&color_attachments));

            render_pass.set_pipeline(&my_gfx.pipeline.pipeline);
            render_pass.set_bind_group(0, &my_gfx.camera.uniform.bind_group, &[]);

            render_pass.set_bind_group(1, &my_gfx.cube.bind_group, &[]);
            render_pass.set_vertex_buffer(0, my_gfx.vertex_buffer.buffer.slice(..));
            render_pass.set_index_buffer(my_gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..etib::INDICES.len() as u32, 0, 0..1);

            render_pass.set_bind_group(1, &my_gfx.cube2.bind_group, &[]);
            render_pass.draw_indexed(0..etib::INDICES.len() as u32, 0, 0..1);
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
            WindowEvent::Resized(size) => {
                if let Some(gfx) = &mut self.gfx {
                    gfx.reconfigure_surface_size(size);

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
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Initializing ETIB");

    let mut game = MyGame::default();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    etib::run(&mut game, event_loop)?;

    Ok(())
}
