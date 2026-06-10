use std::sync::Arc;

use etib::Game;
use glam::Vec3;
use winit::application::ApplicationHandler;
use winit::event::*;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode::*, PhysicalKey};
use winit::platform::macos::WindowAttributesExtMacOS;
use winit::window::{Fullscreen, Window, WindowId};

use etib::raytracing::InputState;
use etib::raytracing::camera::Camera;
use etib::raytracing::cube::{Cube, Material, World};
use etib::raytracing::rasterizer::RasterizerPass;
use etib::raytracing::raytracing::{RaytracingPass, RenderPass as DisplayPass};
use etib::raytracing::renderer::Renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = RaytracingExample::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[derive(Default)]
struct RaytracingExample {
    renderer: Option<Renderer>,
    rasterizing_pass: Option<RasterizerPass>,
    raytracing_pass: Option<RaytracingPass>,
    display_pass: Option<DisplayPass>,
    window: Option<Arc<Window>>,

    last_frame_time: Option<std::time::Instant>,
    fps: f32,
    frame_count: u32,

    camera: Camera,
    input_state: InputState,
}

impl Game for RaytracingExample {
    type InitParams = ();

    fn init(ctx: &mut etib::EngineContext, params: Self::InitParams) -> Self {
        todo!()
    }

    fn update(&mut self, ctx: &mut etib::EngineContext) {
        todo!()
    }
}

impl RaytracingExample {
    fn get_window(&self) -> Arc<Window> {
        self.window.as_ref().unwrap().clone()
    }

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let window = self.get_window();
        let renderer = Renderer::new(window.clone());

        let world = World::new(&[
            Cube::new(
                Vec3::new(0.0, 0.0, -10.0),
                0.5,
                Material::Lambertian(Vec3::new(1.0, 0.0, 0.0)),
            ),
            Cube::new(
                Vec3::new(0.0, 1.0, -10.0),
                0.5,
                Material::Lambertian(Vec3::new(1.0, 1.0, 0.0)),
            ),
            Cube::new(
                Vec3::new(1.0, 1.0, -10.0),
                0.5,
                Material::Lambertian(Vec3::new(1.0, 0.0, 1.0)),
            ),
            Cube::new(
                Vec3::new(1.0, 0.0, -10.0),
                0.5,
                Material::Metal(Vec3::new(0.8, 0.3, 0.3), 0.1),
            ),
            Cube::new(Vec3::new(0.0, -1.0, -10.0), 0.5, Material::Dielectric(1.33)),
            Cube::new(
                Vec3::new(1.0, -1.0, -10.0),
                0.5,
                Material::Metal(Vec3::new(0.0, 0.0, 1.0), 0.3),
            ),
            Cube::new(
                Vec3::new(0.0, 100000.0, 0.0),
                0.5,
                Material::Emissive(Vec3::new(300.0, 100.0, 0.0)),
            ),
            Cube::new(
                Vec3::new(0.0, -101.5, 0.0),
                100.0,
                // Material::Lambertian(Vec3::new(0.0, 0.1, 0.1)),
                Material::Metal(Vec3::new(0.5, 0.5, 0.5), 0.1),
            ),
        ]);

        let skybox_bytes = std::fs::read("./sky.hdr").expect("Failed to read skybox file");
        let skybox_texture = renderer.create_skybox_texture("Skybox", &skybox_bytes);

        let rasterizing_pass = RasterizerPass::new(&renderer, &world);
        let raytracing_pass = RaytracingPass::new(
            &renderer,
            &world,
            &rasterizing_pass.albedo_texture,
            &rasterizing_pass.normal_texture,
            &rasterizing_pass.depth_texture,
            &rasterizing_pass.material_texture,
            &skybox_texture,
        );
        let display_pass = DisplayPass::new(&renderer, raytracing_pass.texture());

        self.last_frame_time = Some(std::time::Instant::now());

        self.renderer = Some(renderer);
        self.rasterizing_pass = Some(rasterizing_pass);
        self.raytracing_pass = Some(raytracing_pass);
        self.display_pass = Some(display_pass);

        Ok(())
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let renderer = self.renderer.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();

        // FPS Calculation & Camera Update
        if let Some(last_time) = self.last_frame_time {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_time).as_secs_f32();
            if elapsed > 0.0 {
                self.fps = 1.0 / elapsed;
                window.set_title(&format!("Ray Tracing - {:.1} FPS", self.fps));
            }
            self.last_frame_time = Some(now);

            let speed = 5.0 * elapsed;
            self.camera
                .update(&self.input_state, speed, &mut self.frame_count);
        }

        self.rasterizing_pass.as_ref().unwrap().update(
            renderer,
            &self.camera,
            &mut self.frame_count,
        );

        self.raytracing_pass.as_ref().unwrap().update(
            renderer,
            &self.camera,
            &mut self.frame_count,
        );

        renderer.render(
            window,
            self.rasterizing_pass.as_ref().unwrap(),
            self.raytracing_pass.as_ref().unwrap(),
            self.display_pass.as_ref().unwrap(),
        )?;

        Ok(())
    }
}

impl ApplicationHandler for RaytracingExample {
    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if self.input_state.rmb_pressed {
                self.camera.yaw += (delta.0 as f32) * 0.005;
                self.camera.pitch -= (delta.1 as f32) * 0.005;
                let clamp_val = 89.0f32.to_radians();
                self.camera.pitch = self.camera.pitch.clamp(-clamp_val, clamp_val);
                self.frame_count = 0;
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Ray Tracing")
                            .with_fullscreen(Some(Fullscreen::Borderless(None)))
                            .with_borderless_game(true),
                    )
                    .unwrap(),
            ));

            self.init().unwrap();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                let is_pressed = state == ElementState::Pressed;
                match key_code {
                    KeyW => self.input_state.forward = is_pressed,
                    KeyS => self.input_state.backward = is_pressed,
                    KeyA => self.input_state.left = is_pressed,
                    KeyD => self.input_state.right = is_pressed,
                    Space => self.input_state.up = is_pressed,
                    ShiftLeft => self.input_state.down = is_pressed,
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right {
                    self.input_state.rmb_pressed = state == ElementState::Pressed;
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                match self.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                };
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => {}
        }
    }
}
