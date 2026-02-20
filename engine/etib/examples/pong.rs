use std::f32::consts::FRAC_PI_4;

use cgmath::Vector3;
use winit::dpi::PhysicalSize;
use winit::event::DeviceEvent;
use winit::keyboard::KeyCode;

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
const SPEED_BOOST_MULTIPLIER: f32 = 2.0;
const SPEED_BOOST_DURATION: f32 = 2.0;
const SPEED_BOOST_COOLDOWN: f32 = 15.0;
/// Maximum paddle center Y so the paddle never clips through a wall.
/// Wall inner face: FIELD_HALF_H − 0.5. Outermost paddle cube face: center + PADDLE_HALF_H + 0.5.
const PADDLE_MAX_Y: f32 = FIELD_HALF_H - PADDLE_HALF_H - 1.0;

// ---------------------------------------------------------------------------
// Ball
// ---------------------------------------------------------------------------
const BALL_SPEED_INIT: f32 = 10.0;
const BALL_SPEED_MAX: f32 = 40.0;
const BALL_SPEED_INC: f32 = 1.5;
const BALL_GRAVITY: f32 = 50.0;
const BALL_LOB_SPEED: f32 = 30.0;
const BALL_LOB_X_FACTOR: f32 = 0.9;
/// Ball center Y limit so it never clips into a wall (wall face − ball radius).
const WALL_LIMIT: f32 = FIELD_HALF_H - 1.0;

// ---------------------------------------------------------------------------
// Camera  — perspective from a low angle to reveal cube depth
// ---------------------------------------------------------------------------
const CAM_EYE: (f32, f32, f32) = (0.0, -30.0, 20.0);
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
// 3D Score Rendering
// ---------------------------------------------------------------------------

fn make_digit_cubes(digit: char, offset_x: f32, color: Vector3<f32>) -> Vec<ModelCube> {
    let mut cubes = Vec::new();
    let map = match digit {
        '0' => [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1],
        '1' => [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0],
        '2' => [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1],
        '3' => [1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1],
        '4' => [1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1],
        '5' => [1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1],
        '6' => [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1],
        '7' => [1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1],
        '8' => [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1],
        '9' => [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1],
        _ => [0; 15],
    };

    for y in 0..5 {
        for x in 0..3 {
            if map[(4 - y) * 3 + x] == 1 {
                // Digits drawn in X/Y plane in the background (-15.0 on Z)
                cubes.push(unit_cube(
                    offset_x + x as f32,
                    y as f32,
                    -15.0,
                    color.x,
                    color.y,
                    color.z,
                ));
            }
        }
    }
    cubes
}

fn make_score_model(score: u32, base_x: f32, color: Vector3<f32>) -> DynamicModel {
    let s = score.to_string();
    let mut cubes = Vec::new();
    let mut offset_x = base_x;
    for c in s.chars() {
        cubes.extend(make_digit_cubes(c, offset_x, color));
        offset_x += 4.0; // Spacing between digits
    }
    let mut model = DynamicModel::from_cubes(cubes);
    // Raise the score board
    model.position = Vector3::new(0.0, FIELD_HALF_H + 2.0, 0.0);
    model
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

    // Paddle centre positions
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,

    // Ball Z-axis mapping (lobbing)
    ball_z: f32,
    ball_vel_z: f32,

    // Speed boost state
    left_boost_active: bool,
    right_boost_active: bool,
    left_boost_timer: f32,
    right_boost_timer: f32,
    left_cooldown_timer: f32,
    right_cooldown_timer: f32,

    // Score
    left_score: u32,
    right_score: u32,
    rendered_left_score: i32,
    rendered_right_score: i32,

    // Dynamic model IDs
    ball_id: usize,
    left_id: usize,
    right_id: usize,
    left_score_id: Option<usize>,
    right_score_id: Option<usize>,

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
        ctx.set_window_title("Pong 3D");
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

        let left_color = Vector3::new(0.2, 0.75, 1.0);
        let right_color = Vector3::new(1.0, 0.45, 0.1);

        let walls = make_static_geometry();
        let left_paddle = make_paddle(left_color.x, left_color.y, left_color.z);
        let right_paddle = make_paddle(right_color.x, right_color.y, right_color.z);
        let ball = make_ball();

        // 3D Scoreboards
        let left_score_model = make_score_model(0, -10.0, left_color);
        let right_score_model = make_score_model(0, 10.0, right_color);

        let max_dyn =
            left_paddle.cube_count() + right_paddle.cube_count() + ball.cube_count() + 100; // room for score numbers

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
        let left_score_id = scene.add_dynamic(left_score_model);
        let right_score_id = scene.add_dynamic(right_score_model);

        scene.get_dynamic_mut(left_id).unwrap().position = Vector3::new(-PADDLE_X, 0.0, 0.0);
        scene.get_dynamic_mut(right_id).unwrap().position = Vector3::new(PADDLE_X, 0.0, 0.0);

        PongGame {
            my_gfx: Some(PongGfx { camera, scene }),
            ball_x: 0.0,
            ball_y: 0.0,
            ball_vel_x: BALL_SPEED_INIT,
            ball_vel_y: BALL_SPEED_INIT * 0.4,
            left_x: -PADDLE_X,
            left_y: 0.0,
            right_x: PADDLE_X,
            right_y: 0.0,
            ball_z: 0.0,
            ball_vel_z: 0.0,
            left_boost_active: false,
            right_boost_active: false,
            left_boost_timer: 0.0,
            right_boost_timer: 0.0,
            left_cooldown_timer: 0.0,
            right_cooldown_timer: 0.0,
            left_score: 0,
            right_score: 0,
            rendered_left_score: 0,
            rendered_right_score: 0,
            ball_id,
            left_id,
            right_id,
            left_score_id: Some(left_score_id),
            right_score_id: Some(right_score_id),
            frame_count: 0,
            fps_timer: 0.0,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.time.dt;

        // --- Speed Boost Logic ---
        if self.left_cooldown_timer > 0.0 {
            self.left_cooldown_timer -= dt;
        }
        if self.right_cooldown_timer > 0.0 {
            self.right_cooldown_timer -= dt;
        }

        if self.left_boost_timer > 0.0 {
            self.left_boost_timer -= dt;
            if self.left_boost_timer <= 0.0 {
                self.left_boost_active = false;
            }
        }
        if self.right_boost_timer > 0.0 {
            self.right_boost_timer -= dt;
            if self.right_boost_timer <= 0.0 {
                self.right_boost_active = false;
            }
        }

        if ctx.input.is_key_just_pressed(KeyCode::ControlLeft) && self.left_cooldown_timer <= 0.0 {
            self.left_boost_active = true;
            self.left_boost_timer = SPEED_BOOST_DURATION;
            self.left_cooldown_timer = SPEED_BOOST_COOLDOWN;
        }
        if ctx.input.is_key_just_pressed(KeyCode::Enter) && self.right_cooldown_timer <= 0.0 {
            self.right_boost_active = true;
            self.right_boost_timer = SPEED_BOOST_DURATION;
            self.right_cooldown_timer = SPEED_BOOST_COOLDOWN;
        }

        let left_speed = if self.left_boost_active {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };
        let right_speed = if self.right_boost_active {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };

        // --- Paddle movement ---
        // Left paddle (W/S for Y, A/D for X)
        if ctx.input.is_key_pressed(KeyCode::KeyW) {
            self.left_y = (self.left_y + left_speed * dt).min(PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::KeyS) {
            self.left_y = (self.left_y - left_speed * dt).max(-PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            self.left_x = (self.left_x + left_speed * dt).min(-2.0); // Don't cross centre
        }
        if ctx.input.is_key_pressed(KeyCode::KeyA) {
            self.left_x = (self.left_x - left_speed * dt).max(-FIELD_HALF_W + PADDLE_HALF_H);
        }

        // Right paddle (Up/Down for Y, Left/Right for X)
        if ctx.input.is_key_pressed(KeyCode::ArrowUp) {
            self.right_y = (self.right_y + right_speed * dt).min(PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowDown) {
            self.right_y = (self.right_y - right_speed * dt).max(-PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowLeft) {
            self.right_x = (self.right_x - right_speed * dt).max(2.0); // Don't cross centre
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowRight) {
            self.right_x = (self.right_x + right_speed * dt).min(FIELD_HALF_W - PADDLE_HALF_H);
        }

        // --- Ball movement ---
        self.ball_x += self.ball_vel_x * dt;
        self.ball_y += self.ball_vel_y * dt;
        self.ball_z += self.ball_vel_z * dt;

        // Apply gravity
        self.ball_vel_z -= BALL_GRAVITY * dt;

        // Floor bounce / clamp
        if self.ball_z <= 0.0 {
            self.ball_z = 0.0;
            // Simple bounce if falling fast, or land
            if self.ball_vel_z < -5.0 {
                self.ball_vel_z *= -0.5;
            } else {
                self.ball_vel_z = 0.0;
            }
        }

        // --- Top / bottom wall bounce ---
        if self.ball_y > WALL_LIMIT {
            self.ball_y = WALL_LIMIT;
            self.ball_vel_y = -self.ball_vel_y.abs();
        } else if self.ball_y < -WALL_LIMIT {
            self.ball_y = -WALL_LIMIT;
            self.ball_vel_y = self.ball_vel_y.abs();
        }

        // --- Collision logic ---
        // Ball can only hit paddles if it is low enough (e.g. not lobbed over them)
        let ball_is_hittable = self.ball_z < 3.0;

        // Left paddle collision
        let left_contact = self.left_x + 1.0;
        if ball_is_hittable
            && self.ball_vel_x < 0.0
            && self.ball_x <= left_contact
            && self.ball_x > self.left_x - 2.0 // prevent tunneling
            && (self.ball_y - self.left_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = left_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.left_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            if ctx.input.is_key_pressed(KeyCode::ShiftLeft) {
                self.ball_vel_z = BALL_LOB_SPEED; // lob it
                self.ball_vel_x = (new_speed * BALL_LOB_X_FACTOR) * angle.cos(); // slower x
            } else {
                self.ball_vel_x = new_speed * angle.cos();
            }
            self.ball_vel_y = new_speed * angle.sin();
        }

        // Right paddle collision
        let right_contact = self.right_x - 1.0;
        if ball_is_hittable
            && self.ball_vel_x > 0.0
            && self.ball_x >= right_contact
            && self.ball_x < self.right_x + 2.0 // prevent tunneling
            && (self.ball_y - self.right_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = right_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.right_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            if ctx.input.is_key_pressed(KeyCode::ShiftRight) {
                self.ball_vel_z = BALL_LOB_SPEED;
                self.ball_vel_x = -(new_speed * BALL_LOB_X_FACTOR) * angle.cos();
            } else {
                self.ball_vel_x = -new_speed * angle.cos();
            }
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

        // Rebuild 3D score models if they changed
        let scene = &mut self.my_gfx.as_mut().unwrap().scene;
        if self.left_score as i32 != self.rendered_left_score {
            if let Some(old_id) = self.left_score_id {
                scene.remove_dynamic(old_id);
            }
            let left_color = Vector3::new(0.2, 0.75, 1.0);
            let left_score_model = make_score_model(self.left_score, -10.0, left_color);
            self.left_score_id = Some(scene.add_dynamic(left_score_model));
            self.rendered_left_score = self.left_score as i32;
        }

        if self.right_score as i32 != self.rendered_right_score {
            if let Some(old_id) = self.right_score_id {
                scene.remove_dynamic(old_id);
            }
            let right_color = Vector3::new(1.0, 0.45, 0.1);
            let right_score_model = make_score_model(self.right_score, 10.0, right_color);
            self.right_score_id = Some(scene.add_dynamic(right_score_model));
            self.rendered_right_score = self.right_score as i32;
        }

        // --- Sync GPU transforms ---
        let scene = &mut self.my_gfx.as_mut().unwrap().scene;
        scene.get_dynamic_mut(self.ball_id).unwrap().position =
            Vector3::new(self.ball_x, self.ball_y, self.ball_z);
        scene.get_dynamic_mut(self.left_id).unwrap().position =
            Vector3::new(self.left_x, self.left_y, 0.0);
        scene.get_dynamic_mut(self.right_id).unwrap().position =
            Vector3::new(self.right_x, self.right_y, 0.0);
    }

    fn ui(&mut self, _ctx: &mut EngineContext, ui_ctx: &etib::egui::Context) {
        etib::egui::Area::new(etib::egui::Id::new("pong_debug_info")).show(ui_ctx, |ui| {
            ui.with_layout(
                etib::egui::Layout::left_to_right(etib::egui::Align::Center),
                |ui| {
                    if ui.button("Randomize Ball").clicked() {
                        let t = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64();
                        let r = (t * 13.0).sin().abs() as f32;
                        let g = (t * 17.0).sin().abs() as f32;
                        let b = (t * 19.0).sin().abs() as f32;
                        if let Some(my_gfx) = self.my_gfx.as_mut() {
                            if let Some(ball) = my_gfx.scene.get_dynamic_mut(self.ball_id) {
                                for cube in ball.cubes_mut() {
                                    cube.color = cgmath::Vector3::new(r, g, b);
                                }
                            }
                        }
                    }

                    ui.separator();
                    ui.heading("Boosts:");
                    ui.label(format!(
                        "Left Cooldown: {:.1}s",
                        self.left_cooldown_timer.max(0.0)
                    ));
                    ui.label(format!(
                        "Right Cooldown: {:.1}s",
                        self.right_cooldown_timer.max(0.0)
                    ));

                    ui.separator();
                    ui.heading("Ball Info:");
                    ui.label(format!("Speed X: {:.2}", self.ball_vel_x));
                    ui.label(format!("Speed Y: {:.2}", self.ball_vel_y));

                    ui.separator();
                    ui.label(format!(
                        "FPS: {:.0}",
                        self.frame_count as f32 / self.fps_timer.max(0.001)
                    ));
                },
            );
        });
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        let (frame, view) = ctx.gfx.get_next_frame();

        self.frame_count += 1;
        self.fps_timer += ctx.time.dt;
        if self.fps_timer >= 0.5 {
            self.frame_count = 0;
            self.fps_timer = 0.0;
        }

        let mut encoder = ctx
            .gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Pong encoder"),
            });

        let my_gfx = self.my_gfx.as_mut().unwrap();
        my_gfx.scene.render(
            &mut encoder,
            &ctx.gfx,
            &view,
            &my_gfx.camera.bind_group.bind_group,
        );

        // `ctx.window` isn't accessible, we changed render_ui to implicitly use it from context instead or we should export window access. Wait, game.rs `render_ui` expects `&Window`?
        ctx.render_ui(&mut encoder, &view);

        ctx.gfx.queue.submit(Some(encoder.finish()));
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

    fn device_input(&mut self, _ctx: &mut EngineContext, _event: &DeviceEvent) {}
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = EngineConfig::load_from_file("config.json");
    etib::run::<PongGame>(config, Some(()))?;
    Ok(())
}
