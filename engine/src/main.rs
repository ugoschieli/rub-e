use crate::camera::Camera;
use crate::chunk::World;
use crate::cube::Cube;
use crate::gfx::Gfx;
use crate::time::Time;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

mod camera;
mod chunk;
mod cube;
mod frame_buffer;
mod gfx;
mod time;
mod utils;

const CUBE_NUMBER: usize = 1_000_000;
const CUBE_RANGE: Range<i32> = -128..128;

#[derive(Debug)]
struct App {
    window: Option<Arc<Window>>,
    window_size: PhysicalSize<u32>,
    gfx: Option<Gfx>,
    camera: Option<Camera>,
    keys_held: HashSet<KeyCode>,
    time: Time,
    cubes: Vec<Cube>,
    world: Option<World>,
}

impl App {
    pub fn new() -> Self {
        let cubes = (0..CUBE_NUMBER)
            .map(|_| Cube::random_cube(CUBE_RANGE))
            .collect::<Vec<Cube>>();

        Self {
            window: None,
            window_size: PhysicalSize::new(0, 0),
            gfx: None,
            camera: None,
            keys_held: HashSet::default(),
            time: Time::new(),
            cubes,
            world: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("ETIB")
                        .with_fullscreen(Some(Fullscreen::Borderless(None))),
                )
                .expect("Failed to create a window"),
        );

        window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        window.set_cursor_visible(false);

        self.gfx = Some(Gfx::new(event_loop, window.clone(), self));
        self.window = Some(window);
        self.camera = Some(Camera::new());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.window_size = new_size;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys_held.insert(code);
                    } else {
                        self.keys_held.remove(&code);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.time.tick();
                let camera = self.camera.as_mut().unwrap();
                camera.handle_keyboard(&self.keys_held, &self.time);

                let gfx = self.gfx.as_mut().unwrap();
                let window = self.window.as_ref().unwrap();

                gfx.render(
                    camera,
                    self.window_size,
                    &self.time,
                    self.world.as_ref().unwrap(),
                );

                window.request_redraw();
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let camera = self.camera.as_mut().unwrap();

        if let DeviceEvent::MouseMotion { delta } = event {
            camera.handle_mouse(delta);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
