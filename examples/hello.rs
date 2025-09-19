use env_logger;
use etib::Game;
use log::info;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

#[derive(Default)]
struct MyGame {
    window: Option<Arc<Window>>,
    gfx: Option<etib::Gfx>,
    time: etib::TimeState,
    is_initialized: bool,
}

impl etib::Game for MyGame {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "ETIB".to_owned();
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let gfx = etib::Gfx::new(window.clone());

        self.window = Some(window);
        self.gfx = Some(gfx);
        self.is_initialized = true;
    }

    fn render(&mut self) {
        let gfx = self.gfx();
        let (frame, view) = gfx.get_next_frame();

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let color_attachments = [Some(etib::Gfx::color_attachments_from_view(&view))];
            let mut render_pass = encoder.begin_render_pass(&gfx.render_pass(&color_attachments));

            render_pass.set_pipeline(&gfx.pipeline.pipeline);
            render_pass.set_bind_group(0, &gfx.camera.uniform.bind_group, &[]);
            render_pass.set_vertex_buffer(0, gfx.vertex_buffer.slice(..));
            render_pass.set_index_buffer(gfx.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..etib::INDICES.len() as u32, 0, 0..1);
            // render_pass.draw(0..vertex::VERTICES2.len() as u32, 0..1);
        }

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

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

impl ApplicationHandler for MyGame {
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
