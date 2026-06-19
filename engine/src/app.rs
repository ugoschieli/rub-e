use crate::camera::Camera;
use crate::constants::{CUBE_NUMBER, CUBE_RANGE};
use crate::cube::Cube;
use crate::gfx::Gfx;
use crate::renderer::Renderer;
use crate::renderer::dynamic_renderer::DynamicRenderer;
use crate::renderer::static_renderer::StaticRenderer;
use crate::time::Time;
use crate::updater::orbit_updater::OrbitUpdater;
use crate::updater::line_updater::LineUpdater;
use crate::updater::{TransformBuffers, Updater};
use std::collections::HashSet;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

#[derive(Debug)]
pub struct App {
    pub window: Option<Arc<Window>>,
    pub window_size: PhysicalSize<u32>,
    pub gfx: Option<Gfx>,
    pub camera: Option<Camera>,
    pub keys_held: HashSet<KeyCode>,
    pub time: Time,
    pub cubes: Vec<Cube>,
    pub transforms: Option<TransformBuffers>,
    pub updaters: Vec<Box<dyn Updater>>,
    pub renderers: Vec<Box<dyn Renderer>>,
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
            transforms: None,
            updaters: vec![],
            renderers: vec![],
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
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

        let gfx = Gfx::new(event_loop, window.clone(), self);
        let camera = Camera::new(&gfx);

        // Shared transform buffers the updaters write and the renderer reads.
        let transforms = TransformBuffers::new(&gfx, &self.cubes);

        // Split the cubes in half: the first half orbits, the second oscillates
        // along a line. Each updater writes its own contiguous transform range.
        let half = self.cubes.len() / 2;
        let orbit = OrbitUpdater::init(&gfx, &self.cubes[..half], transforms.buffers(), 0);
        let line = LineUpdater::init(
            &gfx,
            &self.cubes[half..],
            transforms.buffers(),
            half as u32,
        );

        let renderer = StaticRenderer::init(&gfx, &camera);
        let dynamic = DynamicRenderer::init(&gfx, &camera, &self.cubes, transforms.buffers());

        self.gfx = Some(gfx);
        self.window = Some(window);
        self.camera = Some(camera);
        self.transforms = Some(transforms);
        self.updaters.push(Box::new(orbit));
        self.updaters.push(Box::new(line));
        self.renderers.push(Box::new(renderer));
        // Pushed after StaticRenderer so it loads/composites over the cleared pass.
        self.renderers.push(Box::new(dynamic));
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

                let gfx = self.gfx.as_mut().unwrap();
                let camera = self.camera.as_mut().unwrap();
                let window = self.window.as_ref().unwrap();

                camera.handle_keyboard(&self.keys_held, &self.time);

                gfx.update(&self.time);

                if gfx.surface_texture.is_some() {
                    camera.upload(gfx, self.window_size);

                    // Updaters write this frame's transforms before any renderer
                    // reads them (the pass boundary is the memory barrier).
                    for updater in &mut self.updaters {
                        updater.update(gfx);
                    }

                    for renderer in &mut self.renderers {
                        renderer.render(gfx, camera, self.window_size);
                    }

                    gfx.submit();
                }

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
