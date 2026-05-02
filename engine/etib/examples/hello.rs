use std::path::Path;

use cgmath::{InnerSpace, Rotation3};
use clap::Parser;
use log::info;
use winit::event::{DeviceEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::config::EngineConfig;
use etib::{EngineContext, Game};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Engine configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[arg(long)]
    isometric: bool,

    /// Model file to render as static geometry.
    model_root: String,
}

struct MyParams {
    model_root: String,
    is_isometric: bool,
}

struct MyGame {
    scene: etib::Scene,
    camera_controller: etib::camera::CameraController,
    cursor_grabbed: bool,
    frame_count: u32,
    fps_update_timer: f32,
    time_elapsed: f32,
    orbiting_id: usize,
}

impl Game for MyGame {
    type InitParams = MyParams;

    fn init(ctx: &mut EngineContext, params: Self::InitParams) -> Self {
        let initial_eye = cgmath::Point3::new(0.0_f32, 30.0, 80.0);
        let initial_target = cgmath::Point3::new(0.0_f32, 10.0, 0.0);
        let forward = (initial_target - initial_eye).normalize();
        let initial_yaw: f32 = forward.z.atan2(forward.x);
        let initial_pitch: f32 = forward.y.asin();

        let mut camera_controller = etib::camera::CameraController::new(20.0, 0.003);
        if params.is_isometric {
            camera_controller.mode = etib::camera::CameraMode::Isometric;
        }
        camera_controller.yaw = initial_yaw;
        camera_controller.pitch = initial_pitch;

        let gfx = &ctx.gfx;
        let device = gfx.device();

        let camera = if params.is_isometric {
            etib::camera::Camera::new(
                ctx,
                (50.0, 50.0, 50.0).into(),
                (0.0, 0.0, 0.0).into(),
                cgmath::Vector3::unit_y(),
                ctx.window_size().width as f32 / ctx.window_size().height as f32,
                etib::camera::Projection::Orthographic { scale: 50.0 },
                -200.0,
                200.0,
            )
        } else {
            etib::camera::Camera::new(
                ctx,
                (0.0, 30.0, 80.0).into(),
                (0.0, 10.0, 0.0).into(),
                cgmath::Vector3::unit_y(),
                ctx.window_size().width as f32 / ctx.window_size().height as f32,
                etib::camera::Projection::Perspective { fovy: 45.0 },
                0.1,
                500.0,
            )
        };

        // Static model: load from file, fixed in place.
        let map_path = Path::new(&params.model_root).join("world.model");
        let cat_path = Path::new(&params.model_root).join("cat.model");

        let static_cubes =
            etib::cube::load_model(map_path.to_str().unwrap()).expect("Failed to load model file");

        // Dynamic model: same file, offset and animated each frame.
        let mut orbiting = etib::cube::DynamicModel::load(cat_path.to_str().unwrap())
            .expect("Failed to load dynamic model");
        orbiting.position = cgmath::Vector3::new(30.0, 0.0, 0.0);

        let mut scene = etib::Scene::new(ctx, camera, &static_cubes, orbiting.cube_count().max(1));
        let orbiting_id = scene.add_dynamic(orbiting);

        scene
            .set_skybox_from_bytes(
                device,
                &gfx.queue,
                include_bytes!("../examples/sky.hdr"),
                1080,
                cgmath::SquareMatrix::identity(),
            )
            .expect("Failed to load skybox");

        MyGame {
            scene,
            camera_controller,
            cursor_grabbed: false,
            frame_count: 0,
            fps_update_timer: 0.0,
            time_elapsed: 0.0,
            orbiting_id,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        self.time_elapsed += ctx.time.dt;
        let t = self.time_elapsed;

        if let Some(m) = self.scene.get_dynamic_mut(self.orbiting_id) {
            m.position.x = (t * 0.5).sin() * 30.0;
            m.position.z = (t * 0.5).cos() * 30.0;
            m.rotation = cgmath::Quaternion::from_angle_y(cgmath::Rad(-t * 0.5));
        }

        self.frame_count += 1;
        self.fps_update_timer += ctx.time.dt;
        if self.fps_update_timer >= 0.5 {
            let fps = self.frame_count as f32 / self.fps_update_timer;
            ctx.set_window_title(&format!(
                "ETIB - {:.0} FPS  |  Static: {} cubes  |  Dynamic: {} cubes",
                fps,
                self.scene.static_count(),
                self.scene.dynamic_count(),
            ));
            self.frame_count = 0;
            self.fps_update_timer = 0.0;
        }

        self.camera_controller
            .update_camera(&ctx.gfx.queue, &mut self.scene.camera, ctx.time.dt);
    }

    fn scene(&mut self) -> Option<&mut etib::Scene> {
        Some(&mut self.scene)
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
    let args = Args::parse();

    info!("Initializing ETIB");

    let config = EngineConfig::load_from_file(args.config.as_deref().unwrap_or("config.json"));

    let params = MyParams {
        is_isometric: args.isometric,
        model_root: args.model_root,
    };

    etib::run::<MyGame>(config, Some(params))?;

    Ok(())
}
