/// Physics demo — 8 coloured cubes fall under gravity and bounce off a checkerboard floor.
///
/// Controls:
///   C        — grab cursor, enable free-look
///   Escape   — release cursor
///   WASD/QE  — fly camera (while cursor is grabbed)
use cgmath::{InnerSpace, Vector3};
use winit::event::DeviceEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::camera::{Camera, CameraController, Projection};
use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::physics::RigidBody;
use etib::{EngineContext, Game, Scene};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Half-size of the ground checkerboard in each XZ direction.
const GROUND_HALF: i32 = 15;

/// Y coordinate of the top surface of the ground layer.
/// Ground cubes are centred at y=0, so their top face is at y=+0.5.
const FLOOR_TOP: f32 = 0.5;

/// Distance from a ball's origin to its bottom face.
/// Each ball is a 2×2×2 block centred at its model origin, extending 1 unit downward.
const BALL_HALF: f32 = 1.0;

const BALL_COUNT: usize = 8;

const COLORS: [(f32, f32, f32); BALL_COUNT] = [
    (1.0, 0.20, 0.20), // red
    (1.0, 0.55, 0.05), // orange
    (1.0, 0.90, 0.10), // yellow
    (0.20, 1.0, 0.20), // green
    (0.10, 0.80, 1.0), // cyan
    (0.20, 0.40, 1.0), // blue
    (0.75, 0.20, 1.0), // purple
    (1.0, 1.0, 1.0),   // white
];

// ---------------------------------------------------------------------------
// Scene construction helpers
// ---------------------------------------------------------------------------

fn make_ground() -> Vec<ModelCube> {
    let mut cubes = Vec::new();
    for x in -GROUND_HALF..GROUND_HALF {
        for z in -GROUND_HALF..GROUND_HALF {
            let bright = ((x + z).rem_euclid(2) == 0) as i32;
            let gray = 0.30 + bright as f32 * 0.35;
            cubes.push(ModelCube {
                position: Vector3::new(x as f32, 0.0, z as f32),
                color: Vector3::new(gray, gray, gray),
            });
        }
    }
    cubes
}

fn make_ball_model(r: f32, g: f32, b: f32) -> DynamicModel {
    // 2×2×2 block, each cube offset by ±0.5 from the model origin
    let mut cubes = Vec::new();
    for xi in 0..2i32 {
        for yi in 0..2i32 {
            for zi in 0..2i32 {
                cubes.push(ModelCube {
                    position: Vector3::new(
                        xi as f32 - 0.5,
                        yi as f32 - 0.5,
                        zi as f32 - 0.5,
                    ),
                    color: Vector3::new(r, g, b),
                });
            }
        }
    }
    DynamicModel::from_cubes(cubes)
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

struct Ball {
    id: usize,
    body: RigidBody,
}

struct PhysicsDemo {
    scene: Scene,
    camera_controller: CameraController,
    cursor_grabbed: bool,
    balls: Vec<Ball>,
}

impl Game for PhysicsDemo {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _: ()) -> Self {
        let size = ctx.window_size();
        let aspect = size.width as f32 / size.height as f32;

        // Camera: straight-on view from in front of the ball column
        let eye = cgmath::Point3::new(0.0_f32, 22.0, 38.0);
        let target = cgmath::Point3::new(0.0, 8.0, 0.0);
        let forward = (target - eye).normalize();

        let mut camera_controller = CameraController::new(15.0, 0.003);
        camera_controller.yaw = forward.z.atan2(forward.x);
        camera_controller.pitch = forward.y.asin();

        let camera = Camera::new(
            ctx,
            eye,
            target,
            Vector3::unit_y(),
            aspect,
            Projection::Perspective { fovy: 50.0 },
            0.1,
            500.0,
        );

        let ground = make_ground();
        // 8 cubes per ball × BALL_COUNT balls
        let max_dynamic = BALL_COUNT * 8;
        let mut scene = Scene::new(ctx, camera, &ground, max_dynamic);

        // Optional skybox — comment out if sky.hdr is unavailable
        scene
            .set_skybox_from_bytes(
                ctx.gfx.device(),
                &ctx.gfx.queue,
                include_bytes!("sky.hdr"),
                1080,
                cgmath::SquareMatrix::identity(),
            )
            .expect("Failed to load skybox");

        // Spawn balls in a line along X, each at a different height
        let mut balls = Vec::new();
        for i in 0..BALL_COUNT {
            let (r, g, b) = COLORS[i];
            let drop_height = 4.0 + i as f32 * 3.5;

            let mut model = make_ball_model(r, g, b);
            model.position = Vector3::new(
                -10.5 + i as f32 * 3.0, // spread along X
                FLOOR_TOP + BALL_HALF + drop_height,
                0.0,
            );

            let id = scene.add_dynamic(model);

            // Higher restitution for later balls → more bouncy
            let restitution = 0.20 + i as f32 * 0.09;
            balls.push(Ball {
                id,
                body: RigidBody::new().with_restitution(restitution),
            });
        }

        PhysicsDemo {
            scene,
            camera_controller,
            cursor_grabbed: false,
            balls,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.dt();

        for ball in &mut self.balls {
            let delta = ball.body.step(dt);
            if let Some(model) = self.scene.get_dynamic_mut(ball.id) {
                model.position += delta;
                ball.body.resolve_floor(&mut model.position, FLOOR_TOP, BALL_HALF);
            }
        }

        self.camera_controller
            .update_camera(&ctx.gfx.queue, &mut self.scene.camera, dt);

        ctx.set_window_title(&format!("Physics Demo — {:.0} FPS", ctx.fps()));
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

fn main() {
    env_logger::init();
    etib::run::<PhysicsDemo>(EngineConfig::default(), Some(())).expect("engine error");
}
