use cgmath::{InnerSpace, Vector3};
use winit::event::DeviceEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::camera::{Camera, CameraController, Projection};
use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::sounds::{SoundManager, strategy::SingleCubeStrategy};
use etib::{EngineContext, Game, Scene};

struct SoundsDemo {
    scene: Scene,
    camera_controller: CameraController,
    cursor_grabbed: bool,
    sound_manager: SoundManager,
    cube_id: usize,
}

impl Game for SoundsDemo {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _: ()) -> Self {
        ctx.set_window_title("Sounds Demo");

        let size = ctx.window_size();
        let aspect = size.width as f32 / size.height as f32;

        let eye = cgmath::Point3::new(0.0_f32, 6.0, 18.0);
        let target = cgmath::Point3::new(0.0, 0.0, 0.0);
        let fwd = (target - eye).normalize();

        let mut camera_controller = CameraController::new(10.0, 0.003);
        camera_controller.yaw = fwd.z.atan2(fwd.x);
        camera_controller.pitch = fwd.y.asin();

        let camera = Camera::new(
            &ctx.gfx.device,
            eye,
            target,
            Vector3::unit_y(),
            aspect,
            Projection::Perspective { fovy: 45.0 },
            0.1,
            500.0,
        );

        let static_cubes: Vec<ModelCube> = Vec::new();
        let mut scene = Scene::new(ctx, camera, &static_cubes, 1);
        scene
            .set_skybox_from_bytes(
                ctx.gfx.device(),
                &ctx.gfx.queue,
                include_bytes!("sky.hdr"),
                1080,
                cgmath::SquareMatrix::identity(),
            )
            .expect("skybox");

        let sound_cube = DynamicModel::from_cubes(vec![ModelCube {
            position: Vector3::new(0.0, 0.0, 0.0),
            color: Vector3::new(1.0, 0.2, 0.2),
        }]);
        let cube_id = scene.add_dynamic(sound_cube);

        let mut sound_manager = SoundManager::new();

        // Resolve path from the crate root, so it works regardless of the process CWD.
        let sound_path = format!("{}/sounds/beep.wav", env!("CARGO_MANIFEST_DIR"));
        sound_manager.add_group(
            vec![Vector3::new(0.0, 0.0, 0.0)],
            &sound_path,
            Box::new(SingleCubeStrategy),
        );

        Self {
            scene,
            camera_controller,
            cursor_grabbed: false,
            sound_manager,
            cube_id,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.dt().min(0.05);
        self.camera_controller
            .update_camera(&ctx.gfx.queue, &mut self.scene.camera, dt);

        // Keep the emitter on the cube's current position.
        if let Some(model) = self.scene.get_dynamic(self.cube_id) {
            // NOTE: SoundManager currently doesn't expose group mutation.
            // For now, we keep the emitter at the initial cube position.
            let _ = model.position;
        }

        self.sound_manager.update(&self.scene.camera);
    }

    fn scene(&mut self) -> Option<&mut Scene> {
        Some(&mut self.scene)
    }

    fn device_input(&mut self, _ctx: &mut EngineContext, event: &DeviceEvent) {
        if self.cursor_grabbed {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.camera_controller.process_mouse(delta.0, delta.1);
            }
        }
    }

    fn input(&mut self, ctx: &mut EngineContext, event: &winit::event::WindowEvent) {
        use winit::event::WindowEvent;
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.camera_controller
                    .process_keyboard(event.physical_key, event.state);
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

fn main() {
    env_logger::init();
    etib::run::<SoundsDemo>(EngineConfig::default(), Some(())).expect("engine error");
}
