use cgmath::{InnerSpace, Rotation3};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::sounds::{SoundManager, strategy::SingleCubeStrategy};
use etib::{EngineContext, Game};

// Position du cube sonore dans le monde
const CUBE_POS: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 0.0);

struct TestGame {
    scene: etib::Scene,
    camera: etib::camera::Camera,
    camera_controller: etib::camera::CameraController,
    cursor_grabbed: bool,
    sound_manager: SoundManager,
    cube_id: usize,
}

impl Game for TestGame {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _params: Self::InitParams) -> Self {
        let gfx = &ctx.gfx;
        let device = gfx.device();

        // Caméra positionnée à distance du cube
        let initial_eye = cgmath::Point3::new(0.0_f32, 5.0, 20.0);
        let initial_target = cgmath::Point3::new(0.0, 0.0, 0.0);
        let forward = (initial_target - initial_eye).normalize();

        let mut camera_controller = etib::camera::CameraController::new(10.0, 0.003);
        camera_controller.yaw = forward.z.atan2(forward.x);
        camera_controller.pitch = forward.y.asin();

        let camera = etib::camera::Camera::new(
            device,
            initial_eye,
            initial_target,
            cgmath::Vector3::unit_y(),
            ctx.window_size().width as f32 / ctx.window_size().height as f32,
            etib::camera::Projection::Perspective { fovy: 45.0 },
            0.1,
            500.0,
        );

        // Un seul cube rouge en (0, 0, 0)
        let cube = ModelCube {
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            color: cgmath::Vector3::new(1.0, 0.2, 0.2),
        };
        let sound_cube = DynamicModel::from_cubes(vec![cube]);

        // Scène sans cubes statiques
        let mut scene = etib::Scene::new(
            gfx,
            ctx.config.peak_brightness_nits,
            &camera.bind_group.layout,
            &[],
            1,
        );
        let cube_id = scene.add_dynamic(sound_cube);

        // Son spatial sur ce cube
        let mut sound_manager = SoundManager::new();
        sound_manager.add_group(
            vec![CUBE_POS],
            "sounds/beep.wav",
            Box::new(SingleCubeStrategy),
        );

        TestGame {
            scene,
            camera,
            camera_controller,
            cursor_grabbed: false,
            sound_manager,
            cube_id,
        }
    }

    fn update(&mut self, _ctx: &mut EngineContext) {
        self.sound_manager.update(&self.camera);
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        let gfx = &ctx.gfx;
        let (frame, view) = gfx.get_next_frame();

        self.camera_controller
            .update_camera(&gfx.queue, &mut self.camera, ctx.time.dt);

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("test encoder"),
            });

        self.scene
            .render(&mut encoder, gfx, &view, &self.camera.bind_group.bind_group);

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, ctx: &mut EngineContext, size: PhysicalSize<u32>) {
        ctx.gfx.reconfigure_surface_size(size);
        self.camera.aspect = size.width as f32 / size.height as f32;
        self.camera.update_matrix(&ctx.gfx.queue);
        self.scene.resize(ctx.gfx.device(), size.width, size.height);
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
    let config = EngineConfig::load_from_file("config.json");
    etib::run::<TestGame>(config, Some(()))?;
    Ok(())
}
