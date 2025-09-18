use env_logger;
use etib::{Game, Gfx, TimeState};
use log::info;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;

struct MyGame {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    time: etib::TimeState,
    is_initialized: bool,
}

impl etib::Game for MyGame {
    fn init(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = Window::default_attributes();
        attributes.title = "ETIB".to_owned();
        let window = Arc::new(event_loop.create_window(attributes).unwrap());

        let gfx = Gfx::new(window.clone());

        self.window = Some(window);
        self.gfx = Some(gfx);
        self.is_initialized = true;
    }

    fn render(&mut self) {
        self.gfx().render();
    }

    fn gfx(&mut self) -> &mut Gfx {
        self.gfx.as_mut().unwrap()
    }

    fn time_state(&self) -> &TimeState {
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

    let mut game = MyGame {
        window: None,
        gfx: None,
        time: TimeState::default(),
        is_initialized: false,
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    etib::run(&mut game, event_loop)?;

    Ok(())
}
