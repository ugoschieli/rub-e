use std::f32::consts::FRAC_PI_4;

use cgmath::Vector3;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::camera::{Camera, Projection};
use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::{EngineContext, Game, Scene};

// ---------------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------------
const FIELD_HALF_W: f32 = 20.0;
const FIELD_HALF_H: f32 = 12.0;

// ---------------------------------------------------------------------------
// Paddles
// ---------------------------------------------------------------------------
const PADDLE_X: f32 = 18.0;
/// Half-height of a paddle in cube units (5 cubes → ±2.5).
const PADDLE_HALF_H: f32 = 2.5;
const PADDLE_SPEED: f32 = 12.0;
/// Maximum paddle center Y so the paddle never clips through a wall.
/// Wall inner face: FIELD_HALF_H − 0.5. Outermost paddle cube face: center + PADDLE_HALF_H + 0.5.
const PADDLE_MAX_Y: f32 = FIELD_HALF_H - PADDLE_HALF_H - 1.0;

// ---------------------------------------------------------------------------
// Ball
// ---------------------------------------------------------------------------
const BALL_SPEED_INIT: f32 = 10.0;
const BALL_SPEED_MAX: f32 = 26.0;
const BALL_SPEED_INC: f32 = 1.5;
/// Ball center Y limit so it never clips into a wall (wall face − ball radius).
const WALL_LIMIT: f32 = FIELD_HALF_H - 1.0;

// ---------------------------------------------------------------------------
// Camera  — perspective from a low angle to reveal cube depth
// ---------------------------------------------------------------------------
const CAM_EYE: (f32, f32, f32) = (0.0, -14.0, 30.0);
const CAM_TARGET: (f32, f32, f32) = (0.0, 1.0, 0.0);
const CAM_FOVY: f32 = 55.0;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unit_cube(x: f32, y: f32, z: f32, r: f32, g: f32, b: f32) -> ModelCube {
    ModelCube {
        position: Vector3::new(x, y, z),
        color: Vector3::new(r, g, b),
    }
}

fn make_paddle(r: f32, g: f32, b: f32) -> DynamicModel {
    let cubes = (-2..=2)
        .map(|i| unit_cube(0.0, i as f32, 0.0, r, g, b))
        .collect();
    DynamicModel::from_cubes(cubes)
}

fn make_ball() -> DynamicModel {
    DynamicModel::from_cubes(vec![unit_cube(0.0, 0.0, 0.0, 1.0, 0.95, 0.2)])
}

/// Static geometry: top wall, bottom wall, and a centre dashed line.
fn make_static_geometry() -> Vec<ModelCube> {
    let mut cubes = Vec::new();

    let xi = -(FIELD_HALF_W as i32)..=(FIELD_HALF_W as i32);
    for x in xi {
        // Top wall
        cubes.push(unit_cube(x as f32, FIELD_HALF_H, 0.0, 0.45, 0.45, 0.45));
        // Bottom wall
        cubes.push(unit_cube(x as f32, -FIELD_HALF_H, 0.0, 0.45, 0.45, 0.45));
    }

    // Centre dashed line (every other Y position)
    let yi = -(FIELD_HALF_H as i32 - 1)..=(FIELD_HALF_H as i32 - 1);
    for y in yi.step_by(2) {
        cubes.push(unit_cube(0.0, y as f32, 0.0, 0.2, 0.2, 0.2));
    }

    cubes
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

struct PongGfx {
    camera: Camera,
    scene: Scene,
}

struct PongGame {
    my_gfx: Option<PongGfx>,

    // Ball
    ball_x: f32,
    ball_y: f32,
    ball_vel_x: f32,
    ball_vel_y: f32,

    // Paddle centre Y positions
    left_y: f32,
    right_y: f32,

    // Held movement keys
    left_up: bool,
    left_down: bool,
    right_up: bool,
    right_down: bool,

    // Score
    left_score: u32,
    right_score: u32,

    // Dynamic model IDs
    ball_id: usize,
    left_id: usize,
    right_id: usize,

    // FPS counter
    frame_count: u32,
    fps_timer: f32,
}

impl PongGame {
    fn current_speed(&self) -> f32 {
        (self.ball_vel_x.powi(2) + self.ball_vel_y.powi(2)).sqrt()
    }

    /// Reset ball to centre and serve toward the player who just lost.
    fn reset_ball(&mut self, toward_right: bool) {
        self.ball_x = 0.0;
        self.ball_y = 0.0;
        let vx = if toward_right {
            BALL_SPEED_INIT
        } else {
            -BALL_SPEED_INIT
        };
        self.ball_vel_x = vx;
        self.ball_vel_y = BALL_SPEED_INIT * 0.4;
    }
}

impl Game for PongGame {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _params: ()) -> Self {
        let gfx = &ctx.gfx;
        let device = gfx.device();

        let aspect = ctx.window_size().width as f32 / ctx.window_size().height as f32;
        let camera = Camera::new(
            device,
            CAM_EYE.into(),
            CAM_TARGET.into(),
            cgmath::Vector3::unit_y(),
            aspect,
            Projection::Perspective { fovy: CAM_FOVY },
            0.1,
            200.0,
        );

        let walls = make_static_geometry();
        let left_paddle = make_paddle(0.2, 0.75, 1.0); // blue
        let right_paddle = make_paddle(1.0, 0.45, 0.1); // orange
        let ball = make_ball();

        let max_dyn = left_paddle.cube_count() + right_paddle.cube_count() + ball.cube_count();

        let mut scene = Scene::new(
            gfx,
            ctx.config.peak_brightness_nits,
            &camera.bind_group.layout,
            &walls,
            max_dyn,
        );

        let ball_id = scene.add_dynamic(ball);
        let left_id = scene.add_dynamic(left_paddle);
        let right_id = scene.add_dynamic(right_paddle);

        scene.get_dynamic_mut(left_id).unwrap().position = Vector3::new(-PADDLE_X, 0.0, 0.0);
        scene.get_dynamic_mut(right_id).unwrap().position = Vector3::new(PADDLE_X, 0.0, 0.0);

        PongGame {
            my_gfx: Some(PongGfx { camera, scene }),
            ball_x: 0.0,
            ball_y: 0.0,
            ball_vel_x: BALL_SPEED_INIT,
            ball_vel_y: BALL_SPEED_INIT * 0.4,
            left_y: 0.0,
            right_y: 0.0,
            left_up: false,
            left_down: false,
            right_up: false,
            right_down: false,
            left_score: 0,
            right_score: 0,
            ball_id,
            left_id,
            right_id,
            frame_count: 0,
            fps_timer: 0.0,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.time.dt;

        // --- Paddle movement ---
        if self.left_up {
            self.left_y = (self.left_y + PADDLE_SPEED * dt).min(PADDLE_MAX_Y);
        }
        if self.left_down {
            self.left_y = (self.left_y - PADDLE_SPEED * dt).max(-PADDLE_MAX_Y);
        }
        if self.right_up {
            self.right_y = (self.right_y + PADDLE_SPEED * dt).min(PADDLE_MAX_Y);
        }
        if self.right_down {
            self.right_y = (self.right_y - PADDLE_SPEED * dt).max(-PADDLE_MAX_Y);
        }

        // --- Ball movement ---
        self.ball_x += self.ball_vel_x * dt;
        self.ball_y += self.ball_vel_y * dt;

        // --- Top / bottom wall bounce ---
        if self.ball_y > WALL_LIMIT {
            self.ball_y = WALL_LIMIT;
            self.ball_vel_y = -self.ball_vel_y.abs();
        } else if self.ball_y < -WALL_LIMIT {
            self.ball_y = -WALL_LIMIT;
            self.ball_vel_y = self.ball_vel_y.abs();
        }

        // --- Left paddle collision ---
        // Left paddle right face X : -PADDLE_X + 0.5
        // Ball left face X         : ball_x − 0.5
        // Touching when ball_x − 0.5 ≤ −PADDLE_X + 0.5  →  ball_x ≤ −PADDLE_X + 1.0
        let left_contact = -PADDLE_X + 1.0;
        if self.ball_vel_x < 0.0
            && self.ball_x <= left_contact
            && self.ball_x > -PADDLE_X - 2.0 // prevent tunneling
            && (self.ball_y - self.left_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = left_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.left_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;
            self.ball_vel_x = new_speed * angle.cos(); // always > 0 (bounces right)
            self.ball_vel_y = new_speed * angle.sin();
        }

        // --- Right paddle collision ---
        // Right paddle left face X : PADDLE_X − 0.5
        // Ball right face X        : ball_x + 0.5
        // Touching when ball_x + 0.5 ≥ PADDLE_X − 0.5  →  ball_x ≥ PADDLE_X − 1.0
        let right_contact = PADDLE_X - 1.0;
        if self.ball_vel_x > 0.0
            && self.ball_x >= right_contact
            && self.ball_x < PADDLE_X + 2.0 // prevent tunneling
            && (self.ball_y - self.right_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = right_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.right_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;
            self.ball_vel_x = -new_speed * angle.cos(); // always < 0 (bounces left)
            self.ball_vel_y = new_speed * angle.sin();
        }

        // --- Scoring ---
        // Serve back toward the player who just missed (right side: right missed, left scored)
        if self.ball_x > FIELD_HALF_W {
            self.left_score += 1;
            self.reset_ball(true); // serve toward right (they just missed)
        } else if self.ball_x < -FIELD_HALF_W {
            self.right_score += 1;
            self.reset_ball(false); // serve toward left (they just missed)
        }

        // --- Sync GPU transforms ---
        let scene = &mut self.my_gfx.as_mut().unwrap().scene;
        scene.get_dynamic_mut(self.ball_id).unwrap().position =
            Vector3::new(self.ball_x, self.ball_y, 0.0);
        scene.get_dynamic_mut(self.left_id).unwrap().position =
            Vector3::new(-PADDLE_X, self.left_y, 0.0);
        scene.get_dynamic_mut(self.right_id).unwrap().position =
            Vector3::new(PADDLE_X, self.right_y, 0.0);
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        let gfx = &ctx.gfx;
        let my_gfx = self.my_gfx.as_mut().unwrap();
        let (frame, view) = gfx.get_next_frame();

        self.frame_count += 1;
        self.fps_timer += ctx.time.dt;
        if self.fps_timer >= 0.5 {
            let fps = self.frame_count as f32 / self.fps_timer;
            ctx.set_window_title(&format!(
                "PONG  |  {}  :  {}  |  {:.0} FPS  |  W/S  vs  Up/Down",
                self.left_score, self.right_score, fps,
            ));
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Pong encoder"),
            });

        my_gfx.scene.render(
            &mut encoder,
            gfx,
            &view,
            &my_gfx.camera.bind_group.bind_group,
        );

        gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, ctx: &mut EngineContext, size: PhysicalSize<u32>) {
        ctx.gfx.reconfigure_surface_size(size);
        let my_gfx = self.my_gfx.as_mut().unwrap();
        my_gfx.camera.aspect = size.width as f32 / size.height as f32;
        my_gfx.camera.update_matrix(&ctx.gfx.queue);
        my_gfx
            .scene
            .resize(ctx.gfx.device(), size.width, size.height);
    }

    fn input(&mut self, _ctx: &mut EngineContext, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            let pressed = event.state == winit::event::ElementState::Pressed;
            match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => self.left_up = pressed,
                PhysicalKey::Code(KeyCode::KeyS) => self.left_down = pressed,
                PhysicalKey::Code(KeyCode::ArrowUp) => self.right_up = pressed,
                PhysicalKey::Code(KeyCode::ArrowDown) => self.right_down = pressed,
                _ => {}
            }
        }
    }

    fn device_input(&mut self, _ctx: &mut EngineContext, _event: &DeviceEvent) {}
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = EngineConfig::load_from_file("config.json");
    etib::run::<PongGame>(config, Some(()))?;
    Ok(())
}
