use clap::Parser;
use std::f32::consts::FRAC_PI_4;
use std::net::UdpSocket;
use std::time::Duration;

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

    // Networking
    client_socket: Option<UdpSocket>,
    server_addr: Option<String>,
    seq_num: u32,
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

        let args = Args::parse();
        let (client_socket, server_addr) = if let Some(ip) = args.client {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client UDP port");
            socket.set_nonblocking(true).unwrap();
            let s_addr = format!("{}:{}", ip, args.port);
            log::info!("Pong Client initialized. Target: {}", s_addr);
            (Some(socket), Some(s_addr))
        } else {
            (None, None)
        };

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
            client_socket,
            server_addr,
            seq_num: 0,
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

        // Apply local logic if NOT connected to a server
        if self.client_socket.is_none() {
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

        // --- Networking Sync ---
        if let (Some(sock), Some(addr)) = (&self.client_socket, &self.server_addr) {
            self.seq_num += 1;

            // Collect Input
            let mut move_y = 0.0;
            let mut move_x = 0.0;
            // W/S or Arrow Keys (Client just sends generic direction and Server decides Left/Right)
            if ctx.input.is_key_pressed(KeyCode::KeyW) || ctx.input.is_key_pressed(KeyCode::ArrowUp)
            {
                move_y = 1.0;
            }
            if ctx.input.is_key_pressed(KeyCode::KeyS)
                || ctx.input.is_key_pressed(KeyCode::ArrowDown)
            {
                move_y = -1.0;
            }
            if ctx.input.is_key_pressed(KeyCode::KeyD)
                || ctx.input.is_key_pressed(KeyCode::ArrowRight)
            {
                move_x = 1.0;
            }
            if ctx.input.is_key_pressed(KeyCode::KeyA)
                || ctx.input.is_key_pressed(KeyCode::ArrowLeft)
            {
                move_x = -1.0;
            }

            let lob_pressed = ctx.input.is_key_pressed(KeyCode::ShiftLeft)
                || ctx.input.is_key_pressed(KeyCode::ShiftRight);
            let boost_pressed = ctx.input.is_key_just_pressed(KeyCode::ControlLeft)
                || ctx.input.is_key_just_pressed(KeyCode::Enter);

            let payload = PlayerInput {
                move_y,
                move_x,
                lob_pressed,
                boost_pressed,
                sequence_number: self.seq_num,
            };

            if let Ok(bytes) = bincode::serialize(&payload) {
                let _ = sock.send_to(&bytes, addr);
            }

            // Receive State Overrides
            let mut buf = [0u8; 1024];
            while let Ok(size) = sock.recv(&mut buf) {
                if let Ok(snap) = bincode::deserialize::<GameStateSnapshot>(&buf[..size]) {
                    self.ball_x = snap.ball_pos[0];
                    self.ball_y = snap.ball_pos[1];
                    self.ball_z = snap.ball_pos[2];
                    self.ball_vel_x = snap.ball_vel[0];
                    self.ball_vel_y = snap.ball_vel[1];
                    self.ball_vel_z = snap.ball_vel[2];

                    self.left_x = snap.left_pos[0];
                    self.left_y = snap.left_pos[1];
                    self.right_x = snap.right_pos[0];
                    self.right_y = snap.right_pos[1];

                    self.left_boost_active = snap.left_boost_active;
                    self.right_boost_active = snap.right_boost_active;

                    self.left_score = snap.left_score;
                    self.right_score = snap.right_score;
                }
            }
        }
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

// ---------------------------------------------------------------------------
// Networking Protocol
// ---------------------------------------------------------------------------
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PlayerInput {
    pub move_y: f32, // -1.0 to 1.0 (Down, None, Up)
    pub move_x: f32, // -1.0 to 1.0 (Left, None, Right)
    pub lob_pressed: bool,
    pub boost_pressed: bool,
    pub sequence_number: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateSnapshot {
    pub ball_pos: [f32; 3], // X, Y, Z
    pub ball_vel: [f32; 3], // X, Y, Z - for client prediction
    pub left_pos: [f32; 2], // X, Y
    pub left_boost_active: bool,
    pub right_pos: [f32; 2], // X, Y
    pub right_boost_active: bool,
    pub left_score: u32,
    pub right_score: u32,
    pub auth_sequence_left: u32,
    pub auth_sequence_right: u32,
}

// ---------------------------------------------------------------------------
// Server Logic
// ---------------------------------------------------------------------------
use std::net::SocketAddr;

struct ServerPlayerState {
    pub addr: SocketAddr,
    pub latest_input: PlayerInput,
    pub last_seq_processed: u32,
    pub cooldown_timer: f32,
    pub boost_timer: f32,
    pub boost_active: bool,
}

struct PongServer {
    socket: UdpSocket,

    // Physics State
    ball_x: f32,
    ball_y: f32,
    ball_z: f32,
    ball_vel_x: f32,
    ball_vel_y: f32,
    ball_vel_z: f32,

    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,

    left_score: u32,
    right_score: u32,

    // Networking State
    left_player: Option<ServerPlayerState>,
    right_player: Option<ServerPlayerState>,
}

impl PongServer {
    fn new(port: u16) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            ball_x: 0.0,
            ball_y: 0.0,
            ball_z: 0.0,
            ball_vel_x: BALL_SPEED_INIT,
            ball_vel_y: BALL_SPEED_INIT * 0.4,
            ball_vel_z: 0.0,
            left_x: -PADDLE_X,
            left_y: 0.0,
            right_x: PADDLE_X,
            right_y: 0.0,
            left_score: 0,
            right_score: 0,
            left_player: None,
            right_player: None,
        })
    }

    fn current_speed(&self) -> f32 {
        (self.ball_vel_x.powi(2) + self.ball_vel_y.powi(2)).sqrt()
    }

    fn reset_ball(&mut self, toward_right: bool) {
        self.ball_x = 0.0;
        self.ball_y = 0.0;
        self.ball_z = 0.0; // reset vertical position
        self.ball_vel_z = 0.0; // reset vertical velocity
        let vx = if toward_right {
            BALL_SPEED_INIT
        } else {
            -BALL_SPEED_INIT
        };
        self.ball_vel_x = vx;
        self.ball_vel_y = BALL_SPEED_INIT * 0.4;
    }

    fn receive_inputs(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, src)) => {
                    if let Ok(input) = bincode::deserialize::<PlayerInput>(&buf[..size]) {
                        // Assignment
                        let is_left = if let Some(p) = &self.left_player {
                            p.addr == src
                        } else {
                            false
                        };
                        let is_right = if let Some(p) = &self.right_player {
                            p.addr == src
                        } else {
                            false
                        };

                        if !is_left && !is_right {
                            if self.left_player.is_none() {
                                log::info!("Left player registered: {}", src);
                                self.left_player = Some(ServerPlayerState {
                                    addr: src,
                                    latest_input: input.clone(),
                                    last_seq_processed: input.sequence_number,
                                    cooldown_timer: 0.0,
                                    boost_timer: 0.0,
                                    boost_active: false,
                                });
                            } else if self.right_player.is_none() {
                                log::info!("Right player registered: {}", src);
                                self.right_player = Some(ServerPlayerState {
                                    addr: src,
                                    latest_input: input.clone(),
                                    last_seq_processed: input.sequence_number,
                                    cooldown_timer: 0.0,
                                    boost_timer: 0.0,
                                    boost_active: false,
                                });
                            }
                        } else {
                            // Update existing
                            if is_left {
                                if let Some(p) = &mut self.left_player {
                                    if input.sequence_number > p.last_seq_processed {
                                        p.latest_input = input;
                                    }
                                }
                            } else if is_right {
                                if let Some(p) = &mut self.right_player {
                                    if input.sequence_number > p.last_seq_processed {
                                        p.latest_input = input;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    log::error!("UDP Recv error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    fn step_physics(&mut self, dt: f32) {
        // Boost timers & logical cooldowns
        if let Some(p) = &mut self.left_player {
            if p.cooldown_timer > 0.0 {
                p.cooldown_timer -= dt;
            }
            if p.boost_timer > 0.0 {
                p.boost_timer -= dt;
                if p.boost_timer <= 0.0 {
                    p.boost_active = false;
                }
            }
            if p.latest_input.boost_pressed && p.cooldown_timer <= 0.0 {
                p.boost_active = true;
                p.boost_timer = SPEED_BOOST_DURATION;
                p.cooldown_timer = SPEED_BOOST_COOLDOWN;
            }
        }
        if let Some(p) = &mut self.right_player {
            if p.cooldown_timer > 0.0 {
                p.cooldown_timer -= dt;
            }
            if p.boost_timer > 0.0 {
                p.boost_timer -= dt;
                if p.boost_timer <= 0.0 {
                    p.boost_active = false;
                }
            }
            if p.latest_input.boost_pressed && p.cooldown_timer <= 0.0 {
                p.boost_active = true;
                p.boost_timer = SPEED_BOOST_DURATION;
                p.cooldown_timer = SPEED_BOOST_COOLDOWN;
            }
        }

        // Apply paddle movements based on latest input
        let left_speed = if self
            .left_player
            .as_ref()
            .map(|p| p.boost_active)
            .unwrap_or(false)
        {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };
        let right_speed = if self
            .right_player
            .as_ref()
            .map(|p| p.boost_active)
            .unwrap_or(false)
        {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };

        if let Some(p) = &self.left_player {
            let input = &p.latest_input;
            if input.move_y > 0.0 {
                self.left_y = (self.left_y + left_speed * dt).min(PADDLE_MAX_Y);
            }
            if input.move_y < 0.0 {
                self.left_y = (self.left_y - left_speed * dt).max(-PADDLE_MAX_Y);
            }
            if input.move_x > 0.0 {
                self.left_x = (self.left_x + left_speed * dt).min(-2.0);
            }
            if input.move_x < 0.0 {
                self.left_x = (self.left_x - left_speed * dt).max(-FIELD_HALF_W + PADDLE_HALF_H);
            }
        }

        if let Some(p) = &self.right_player {
            let input = &p.latest_input;
            if input.move_y > 0.0 {
                self.right_y = (self.right_y + right_speed * dt).min(PADDLE_MAX_Y);
            }
            if input.move_y < 0.0 {
                self.right_y = (self.right_y - right_speed * dt).max(-PADDLE_MAX_Y);
            }
            if input.move_x > 0.0 {
                self.right_x = (self.right_x + right_speed * dt).min(FIELD_HALF_W - PADDLE_HALF_H);
            }
            if input.move_x < 0.0 {
                self.right_x = (self.right_x - right_speed * dt).max(2.0);
            }
        }

        // Ball movement
        self.ball_x += self.ball_vel_x * dt;
        self.ball_y += self.ball_vel_y * dt;
        self.ball_z += self.ball_vel_z * dt;
        self.ball_vel_z -= BALL_GRAVITY * dt; // Gravity

        // Floor bounce
        if self.ball_z <= 0.0 {
            self.ball_z = 0.0;
            if self.ball_vel_z < -5.0 {
                self.ball_vel_z *= -0.5;
            } else {
                self.ball_vel_z = 0.0;
            }
        }

        // Top/bottom bounce
        if self.ball_y > WALL_LIMIT {
            self.ball_y = WALL_LIMIT;
            self.ball_vel_y = -self.ball_vel_y.abs();
        } else if self.ball_y < -WALL_LIMIT {
            self.ball_y = -WALL_LIMIT;
            self.ball_vel_y = self.ball_vel_y.abs();
        }

        // Paddle Collisions (only if low enough)
        let ball_is_hittable = self.ball_z < 3.0;

        let left_contact = self.left_x + 1.0;
        if ball_is_hittable
            && self.ball_vel_x < 0.0
            && self.ball_x <= left_contact
            && self.ball_x > self.left_x - 2.0
            && (self.ball_y - self.left_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = left_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.left_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            let lob_pressed = self
                .left_player
                .as_ref()
                .map(|p| p.latest_input.lob_pressed)
                .unwrap_or(false);
            if lob_pressed {
                self.ball_vel_z = BALL_LOB_SPEED;
                self.ball_vel_x = (new_speed * BALL_LOB_X_FACTOR) * angle.cos();
            } else {
                self.ball_vel_x = new_speed * angle.cos();
            }
            self.ball_vel_y = new_speed * angle.sin();
        }

        let right_contact = self.right_x - 1.0;
        if ball_is_hittable
            && self.ball_vel_x > 0.0
            && self.ball_x >= right_contact
            && self.ball_x < self.right_x + 2.0
            && (self.ball_y - self.right_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball_x = right_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball_y - self.right_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            let lob_pressed = self
                .right_player
                .as_ref()
                .map(|p| p.latest_input.lob_pressed)
                .unwrap_or(false);
            if lob_pressed {
                self.ball_vel_z = BALL_LOB_SPEED;
                self.ball_vel_x = -(new_speed * BALL_LOB_X_FACTOR) * angle.cos();
            } else {
                self.ball_vel_x = -new_speed * angle.cos();
            }
            self.ball_vel_y = new_speed * angle.sin();
        }

        // Scoring
        if self.ball_x > FIELD_HALF_W {
            self.left_score += 1;
            self.reset_ball(true);
        } else if self.ball_x < -FIELD_HALF_W {
            self.right_score += 1;
            self.reset_ball(false);
        }

        // Update sequence numbers
        if let Some(p) = &mut self.left_player {
            p.last_seq_processed = p.latest_input.sequence_number;
        }
        if let Some(p) = &mut self.right_player {
            p.last_seq_processed = p.latest_input.sequence_number;
        }
    }

    fn broadcast_state(&mut self) -> anyhow::Result<()> {
        let snap = GameStateSnapshot {
            ball_pos: [self.ball_x, self.ball_y, self.ball_z],
            ball_vel: [self.ball_vel_x, self.ball_vel_y, self.ball_vel_z],
            left_pos: [self.left_x, self.left_y],
            left_boost_active: self
                .left_player
                .as_ref()
                .map(|p| p.boost_active)
                .unwrap_or(false),
            right_pos: [self.right_x, self.right_y],
            right_boost_active: self
                .right_player
                .as_ref()
                .map(|p| p.boost_active)
                .unwrap_or(false),
            left_score: self.left_score,
            right_score: self.right_score,
            auth_sequence_left: self
                .left_player
                .as_ref()
                .map(|p| p.last_seq_processed)
                .unwrap_or(0),
            auth_sequence_right: self
                .right_player
                .as_ref()
                .map(|p| p.last_seq_processed)
                .unwrap_or(0),
        };

        let bytes = bincode::serialize(&snap)?;

        if let Some(p) = &self.left_player {
            let _ = self.socket.send_to(&bytes, p.addr);
        }
        if let Some(p) = &self.right_player {
            let _ = self.socket.send_to(&bytes, p.addr);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CLI Argument Parsing
// ---------------------------------------------------------------------------
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Act as authoritative server
    #[arg(long, default_value_t = false)]
    server: bool,

    /// IP Address of server to connect to
    #[arg(long)]
    client: Option<String>,

    /// Port to listen (server) or send (client) on
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config = EngineConfig::load_from_file("config.json");
    let args = Args::parse();

    if args.server {
        run_server(args.port)?;
    } else {
        // NOTE: We should handle the client logic later
        etib::run::<PongGame>(config, Some(()))?;
    }

    Ok(())
}

fn run_server(port: u16) -> anyhow::Result<()> {
    log::info!("Starting Headless Pong Server on port {}", port);
    let mut server = PongServer::new(port)?;

    // Simulate at 60Hz
    let dt = 1.0 / 60.0;

    log::info!("Listening for clients...");
    loop {
        server.receive_inputs()?;
        server.step_physics(dt);
        server.broadcast_state()?;
        std::thread::sleep(Duration::from_millis(16));
    }
}
