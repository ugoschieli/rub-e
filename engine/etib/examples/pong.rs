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

    // --- Grass Pitch Floor ---
    let floor_z = -1.0;
    for x in -(FIELD_HALF_W as i32)..=(FIELD_HALF_W as i32) {
        for y in -(FIELD_HALF_H as i32)..=(FIELD_HALF_H as i32) {
            // Create vertical alternating green stripes
            let (r, g, b) = if x % 4 >= -1 && x % 4 <= 1 {
                (0.1, 0.45, 0.1) // Dark green
            } else {
                (0.15, 0.55, 0.15) // Light green
            };
            cubes.push(unit_cube(x as f32, y as f32, floor_z, r, g, b));
        }
    }

    // --- Tennis Net ---
    // At x=0, stretching from -FIELD_HALF_H to +FIELD_HALF_H
    for y in -(FIELD_HALF_H as i32 - 1)..=(FIELD_HALF_H as i32 - 1) {
        for z in 0..=3 {
            // Checkered empty space to look like a net
            if (y + z) % 2 == 0 {
                // Bright white net
                cubes.push(unit_cube(0.0, y as f32, z as f32, 0.95, 0.95, 0.95));
            }
        }
    }

    // --- Stadium Bleachers ---
    let bleacher_rows = 6;

    let color_red = (0.8, 0.2, 0.2);
    let color_blue = (0.2, 0.4, 0.9);
    let color_gray = (0.3, 0.3, 0.3);

    for row in 1..=bleacher_rows {
        let out_y_top = FIELD_HALF_H + row as f32;
        let height_z = row as f32 * 1.0;
        // Top Sideline (Audience)
        for x in -(FIELD_HALF_W as i32 + row)..=(FIELD_HALF_W as i32 + row) {
            let (r, g, b) = if (x + row) % 2 == 0 {
                color_red
            } else {
                color_gray
            };
            cubes.push(unit_cube(x as f32, out_y_top, height_z, r, g, b));
        }

        let out_x_left = -FIELD_HALF_W - row as f32;
        let out_x_right = FIELD_HALF_W + row as f32;

        // Left & Right Goal lines (Audience)
        for y in -(FIELD_HALF_H as i32 + row - 1)..=(FIELD_HALF_H as i32 + row - 1) {
            let (r, g, b) = if (y + row) % 2 == 0 {
                color_blue
            } else {
                color_gray
            };
            cubes.push(unit_cube(out_x_left, y as f32, height_z, r, g, b));

            let (r, g, b) = if (y + row) % 2 == 0 {
                color_red
            } else {
                color_gray
            };
            cubes.push(unit_cube(out_x_right, y as f32, height_z, r, g, b));
        }
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
                cubes.push(unit_cube(
                    offset_x + x as f32,
                    0.0,
                    y as f32 - 4.0,
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
    // Float the score board in the air
    model.position = Vector3::new(0.0, FIELD_HALF_H, 15.0);
    model
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------
struct PongGame {
    scene: Scene,

    //Ball
    ball: Ball,

    // Players
    left_player: Player,
    right_player: Player,

    // Networking
    client_socket: Option<UdpSocket>,
    server_addr: Option<String>,
    seq_num: u32,
}

struct Ball {
    p: Vector3<f32>,
    v: Vector3<f32>,
    id: usize,
}

struct Player {
    x: f32,
    y: f32,
    boost_active: bool,
    boost_timer: f32,
    cooldown_timer: f32,
    score: u32,
    rendered_score: i32,
    id: usize,
    score_id: Option<usize>,
}

impl PongGame {
    fn current_speed(&self) -> f32 {
        (self.ball.v.x.powi(2) + self.ball.v.y.powi(2)).sqrt()
    }

    /// Reset ball to centre and serve toward the player who just lost.
    fn reset_ball(&mut self, toward_right: bool) {
        self.ball.p.x = 0.0;
        self.ball.p.y = 0.0;
        let vx = if toward_right {
            BALL_SPEED_INIT
        } else {
            -BALL_SPEED_INIT
        };
        self.ball.v.x = vx;
        self.ball.v.y = BALL_SPEED_INIT * 0.4;
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

        let ball_model = make_ball();
        let walls = make_static_geometry();
        let left_paddle = make_paddle(left_color.x, left_color.y, left_color.z);
        let right_paddle = make_paddle(right_color.x, right_color.y, right_color.z);
        let left_score_model = make_score_model(0, -10.0, left_color);
        let right_score_model = make_score_model(0, 10.0, right_color);

        let max_dyn =
            left_paddle.cube_count() + right_paddle.cube_count() + ball_model.cube_count() + 100; // room for score numbers

        let mut scene = Scene::new(gfx, camera, &walls, max_dyn);

        let ball = Ball {
            p: Vector3::new(0.0, 0.0, 0.0),
            v: Vector3::new(BALL_SPEED_INIT, BALL_SPEED_INIT * 0.4, 0.0),
            id: scene.add_dynamic(ball_model),
        };

        let left_player = Player {
            x: -PADDLE_X,
            y: 0.0,
            boost_active: false,
            boost_timer: 0.0,
            cooldown_timer: 0.0,
            score: 0,
            rendered_score: 0,
            id: scene.add_dynamic(left_paddle),
            score_id: Some(scene.add_dynamic(left_score_model)),
        };

        let right_player = Player {
            x: PADDLE_X,
            y: 0.0,
            boost_active: false,
            boost_timer: 0.0,
            cooldown_timer: 0.0,
            score: 0,
            rendered_score: 0,
            id: scene.add_dynamic(right_paddle),
            score_id: Some(scene.add_dynamic(right_score_model)),
        };

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

        scene.get_dynamic_mut(left_player.id).unwrap().position = Vector3::new(-PADDLE_X, 0.0, 0.0);
        scene.get_dynamic_mut(right_player.id).unwrap().position = Vector3::new(PADDLE_X, 0.0, 0.0);

        PongGame {
            scene,
            ball,
            left_player,
            right_player,
            client_socket,
            server_addr,
            seq_num: 0,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.time.dt;

        // --- Speed Boost Logic ---
        if self.left_player.cooldown_timer > 0.0 {
            self.left_player.cooldown_timer -= dt;
        }
        if self.right_player.cooldown_timer > 0.0 {
            self.right_player.cooldown_timer -= dt;
        }

        if self.left_player.boost_timer > 0.0 {
            self.left_player.boost_timer -= dt;
            if self.left_player.boost_timer <= 0.0 {
                self.left_player.boost_active = false;
            }
        }
        if self.right_player.boost_timer > 0.0 {
            self.right_player.boost_timer -= dt;
            if self.right_player.boost_timer <= 0.0 {
                self.right_player.boost_active = false;
            }
        }

        if ctx.input.is_key_just_pressed(KeyCode::ControlLeft)
            && self.left_player.cooldown_timer <= 0.0
        {
            self.left_player.boost_active = true;
            self.left_player.boost_timer = SPEED_BOOST_DURATION;
            self.left_player.cooldown_timer = SPEED_BOOST_COOLDOWN;
        }
        if ctx.input.is_key_just_pressed(KeyCode::Enter) && self.right_player.cooldown_timer <= 0.0
        {
            self.right_player.boost_active = true;
            self.right_player.boost_timer = SPEED_BOOST_DURATION;
            self.right_player.cooldown_timer = SPEED_BOOST_COOLDOWN;
        }

        let left_speed = if self.left_player.boost_active {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };
        let right_speed = if self.right_player.boost_active {
            PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
        } else {
            PADDLE_SPEED
        };

        // --- Paddle movement ---
        // Left paddle (W/S for Y, A/D for X)
        if ctx.input.is_key_pressed(KeyCode::KeyW) {
            self.left_player.y = (self.left_player.y + left_speed * dt).min(PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::KeyS) {
            self.left_player.y = (self.left_player.y - left_speed * dt).max(-PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            self.left_player.x = (self.left_player.x + left_speed * dt).min(-2.0); // Don't cross centre
        }
        if ctx.input.is_key_pressed(KeyCode::KeyA) {
            self.left_player.x =
                (self.left_player.x - left_speed * dt).max(-FIELD_HALF_W + PADDLE_HALF_H);
        }

        // Right paddle (Up/Down for Y, Left/Right for X)
        if ctx.input.is_key_pressed(KeyCode::ArrowUp) {
            self.right_player.y = (self.right_player.y + right_speed * dt).min(PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowDown) {
            self.right_player.y = (self.right_player.y - right_speed * dt).max(-PADDLE_MAX_Y);
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowLeft) {
            self.right_player.x = (self.right_player.x - right_speed * dt).max(2.0); // Don't cross centre
        }
        if ctx.input.is_key_pressed(KeyCode::ArrowRight) {
            self.right_player.x =
                (self.right_player.x + right_speed * dt).min(FIELD_HALF_W - PADDLE_HALF_H);
        }

        // --- Ball movement ---
        self.ball.p.x += self.ball.v.x * dt;
        self.ball.p.y += self.ball.v.y * dt;
        self.ball.p.z += self.ball.v.z * dt;

        // Apply local logic if NOT connected to a server
        if self.client_socket.is_none() {
            // Apply gravity
            self.ball.v.z -= BALL_GRAVITY * dt;

            // Floor bounce / clamp
            if self.ball.p.z <= 0.0 {
                self.ball.p.z = 0.0;
                // Simple bounce if falling fast, or land
                if self.ball.v.z < -5.0 {
                    self.ball.v.z *= -0.5;
                } else {
                    self.ball.v.z = 0.0;
                }
            }

            // --- Top / bottom wall bounce ---
            if self.ball.p.y > WALL_LIMIT {
                self.ball.p.y = WALL_LIMIT;
                self.ball.v.y = -self.ball.v.y.abs();
            } else if self.ball.p.y < -WALL_LIMIT {
                self.ball.p.y = -WALL_LIMIT;
                self.ball.v.y = self.ball.v.y.abs();
            }

            // --- Collision logic ---
            // Ball can only hit paddles if it is low enough (e.g. not lobbed over them)
            let ball_is_hittable = self.ball.p.z < 3.0;

            // Left paddle collision
            let left_contact = self.left_player.x + 1.0;
            if ball_is_hittable
                && self.ball.v.x < 0.0
                && self.ball.p.x <= left_contact
                && self.ball.p.x > self.left_player.x - 2.0 // prevent tunneling
                && (self.ball.p.y - self.left_player.y).abs() < PADDLE_HALF_H + 0.5
            {
                self.ball.p.x = left_contact;
                let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
                let offset =
                    ((self.ball.p.y - self.left_player.y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
                let angle = offset * FRAC_PI_4;

                if ctx.input.is_key_pressed(KeyCode::ShiftLeft) {
                    self.ball.v.z = BALL_LOB_SPEED; // lob it
                    self.ball.v.x = (new_speed * BALL_LOB_X_FACTOR) * angle.cos(); // slower x
                } else {
                    self.ball.v.x = new_speed * angle.cos();
                }
                self.ball.v.y = new_speed * angle.sin();
            }

            // Right paddle collision
            let right_contact = self.right_player.x - 1.0;
            if ball_is_hittable
                && self.ball.v.x > 0.0
                && self.ball.p.x >= right_contact
                && self.ball.p.x < self.right_player.x + 2.0 // prevent tunneling
                && (self.ball.p.y - self.right_player.y).abs() < PADDLE_HALF_H + 0.5
            {
                self.ball.p.x = right_contact;
                let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
                let offset =
                    ((self.ball.p.y - self.right_player.y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
                let angle = offset * FRAC_PI_4;

                if ctx.input.is_key_pressed(KeyCode::ShiftRight) {
                    self.ball.v.z = BALL_LOB_SPEED;
                    self.ball.v.x = -(new_speed * BALL_LOB_X_FACTOR) * angle.cos();
                } else {
                    self.ball.v.x = -new_speed * angle.cos();
                }
                self.ball.v.y = new_speed * angle.sin();
            }

            // --- Scoring ---
            // Serve back toward the player who just missed (right side: right missed, left scored)
            if self.ball.p.x > FIELD_HALF_W {
                self.left_player.score += 1;
                self.reset_ball(true); // serve toward right (they just missed)
            } else if self.ball.p.x < -FIELD_HALF_W {
                self.right_player.score += 1;
                self.reset_ball(false); // serve toward left (they just missed)
            }
        }

        // Rebuild 3D score models if they changed
        let scene = &mut self.scene;
        if self.left_player.score as i32 != self.left_player.rendered_score {
            if let Some(old_id) = self.left_player.score_id {
                scene.remove_dynamic(old_id);
            }
            let left_color = Vector3::new(0.2, 0.75, 1.0);
            let left_score_model = make_score_model(self.left_player.score, -10.0, left_color);
            self.left_player.score_id = Some(scene.add_dynamic(left_score_model));
            self.left_player.rendered_score = self.left_player.score as i32;
        }

        if self.right_player.score as i32 != self.right_player.rendered_score {
            if let Some(old_id) = self.right_player.score_id {
                scene.remove_dynamic(old_id);
            }
            let right_color = Vector3::new(1.0, 0.45, 0.1);
            let right_score_model = make_score_model(self.right_player.score, 10.0, right_color);
            self.right_player.score_id = Some(scene.add_dynamic(right_score_model));
            self.right_player.rendered_score = self.right_player.score as i32;
        }

        // --- Sync GPU transforms ---
        let scene = &mut self.scene;
        scene.get_dynamic_mut(self.ball.id).unwrap().position =
            Vector3::new(self.ball.p.x, self.ball.p.y, self.ball.p.z);
        scene.get_dynamic_mut(self.left_player.id).unwrap().position =
            Vector3::new(self.left_player.x, self.left_player.y, 0.0);
        scene
            .get_dynamic_mut(self.right_player.id)
            .unwrap()
            .position = Vector3::new(self.right_player.x, self.right_player.y, 0.0);

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
                    self.ball.p.x = snap.ball_pos[0];
                    self.ball.p.y = snap.ball_pos[1];
                    self.ball.p.z = snap.ball_pos[2];
                    self.ball.v.x = snap.ball_vel[0];
                    self.ball.v.y = snap.ball_vel[1];
                    self.ball.v.z = snap.ball_vel[2];

                    self.left_player.x = snap.left_pos[0];
                    self.left_player.y = snap.left_pos[1];
                    self.right_player.x = snap.right_pos[0];
                    self.right_player.y = snap.right_pos[1];

                    self.left_player.boost_active = snap.left_boost_active;
                    self.right_player.boost_active = snap.right_boost_active;

                    self.left_player.score = snap.left_score;
                    self.right_player.score = snap.right_score;
                }
            }
        }
    }

    fn ui(&mut self, ctx: &mut EngineContext, ui_ctx: &etib::egui::Context) {
        etib::egui::Area::new(etib::egui::Id::new("pong_debug_info")).show(ui_ctx, |ui| {
            ui.with_layout(
                etib::egui::Layout::left_to_right(etib::egui::Align::Center),
                |ui| {
                    if ui.button("Randomize Ball").clicked() {
                        let now = ctx.now();
                        let r = (now * 13.0).sin().abs() as f32;
                        let g = (now * 17.0).sin().abs() as f32;
                        let b = (now * 19.0).sin().abs() as f32;

                        let scene = &mut self.scene;
                        if let Some(ball) = scene.get_dynamic_mut(self.ball.id) {
                            for cube in ball.cubes_mut() {
                                cube.color = cgmath::Vector3::new(r, g, b);
                            }
                        }
                    }

                    ui.separator();
                    ui.heading("Boosts:");
                    ui.label(format!(
                        "Left Cooldown: {:.1}s",
                        self.left_player.cooldown_timer.max(0.0)
                    ));
                    ui.label(format!(
                        "Right Cooldown: {:.1}s",
                        self.right_player.cooldown_timer.max(0.0)
                    ));

                    ui.separator();
                    ui.heading("Ball Info:");
                    ui.label(format!("Speed X: {:.2}", self.ball.v.x));
                    ui.label(format!("Speed Y: {:.2}", self.ball.v.y));

                    ui.separator();
                    ui.label(format!("FPS: {:.0}", ctx.fps()));
                },
            );
        });
    }

    fn render(&mut self, ctx: &mut EngineContext) {
        let (frame, view) = ctx.gfx.get_next_frame();

        let mut encoder = ctx
            .gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Pong encoder"),
            });

        self.scene.render(&mut encoder, &ctx.gfx, &view);

        // `ctx.window` isn't accessible, we changed render_ui to implicitly use it from context instead or we should export window access. Wait, game.rs `render_ui` expects `&Window`?
        ctx.render_ui(&mut encoder, &view);

        ctx.gfx.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn resize(&mut self, ctx: &mut EngineContext, size: PhysicalSize<u32>) {
        ctx.gfx.reconfigure_surface_size(size);
        let scene = &mut self.scene;
        scene.camera.aspect = size.width as f32 / size.height as f32;
        scene.camera.update_matrix(&ctx.gfx.queue);
        scene.resize(ctx.gfx.device(), size.width, size.height);
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
    ball: Ball,

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
        let ball = Ball {
            p: Vector3::new(0.0, 0.0, 0.0),
            v: Vector3::new(BALL_SPEED_INIT, BALL_SPEED_INIT * 0.4, 0.0),
            id: 0,
        };

        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            ball,
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
        (self.ball.v.x.powi(2) + self.ball.v.y.powi(2)).sqrt()
    }

    fn reset_ball(&mut self, toward_right: bool) {
        self.ball.p.x = 0.0;
        self.ball.p.y = 0.0;
        self.ball.p.z = 0.0; // reset vertical position
        self.ball.v.z = 0.0; // reset vertical velocity
        let vx = if toward_right {
            BALL_SPEED_INIT
        } else {
            -BALL_SPEED_INIT
        };
        self.ball.v.x = vx;
        self.ball.v.y = BALL_SPEED_INIT * 0.4;
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
        self.ball.p.x += self.ball.v.x * dt;
        self.ball.p.y += self.ball.v.y * dt;
        self.ball.p.z += self.ball.v.z * dt;
        self.ball.v.z -= BALL_GRAVITY * dt; // Gravity

        // Floor bounce
        if self.ball.p.z <= 0.0 {
            self.ball.p.z = 0.0;
            if self.ball.v.z < -5.0 {
                self.ball.v.z *= -0.5;
            } else {
                self.ball.v.z = 0.0;
            }
        }

        // Top/bottom bounce
        if self.ball.p.y > WALL_LIMIT {
            self.ball.p.y = WALL_LIMIT;
            self.ball.v.y = -self.ball.v.y.abs();
        } else if self.ball.p.y < -WALL_LIMIT {
            self.ball.p.y = -WALL_LIMIT;
            self.ball.v.y = self.ball.v.y.abs();
        }

        // Paddle Collisions (only if low enough)
        let ball_is_hittable = self.ball.p.z < 3.0;

        let left_contact = self.left_x + 1.0;
        if ball_is_hittable
            && self.ball.v.x < 0.0
            && self.ball.p.x <= left_contact
            && self.ball.p.x > self.left_x - 2.0
            && (self.ball.p.y - self.left_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball.p.x = left_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball.p.y - self.left_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            let lob_pressed = self
                .left_player
                .as_ref()
                .map(|p| p.latest_input.lob_pressed)
                .unwrap_or(false);
            if lob_pressed {
                self.ball.v.z = BALL_LOB_SPEED;
                self.ball.v.x = (new_speed * BALL_LOB_X_FACTOR) * angle.cos();
            } else {
                self.ball.v.x = new_speed * angle.cos();
            }
            self.ball.v.y = new_speed * angle.sin();
        }

        let right_contact = self.right_x - 1.0;
        if ball_is_hittable
            && self.ball.v.x > 0.0
            && self.ball.p.x >= right_contact
            && self.ball.p.x < self.right_x + 2.0
            && (self.ball.p.y - self.right_y).abs() < PADDLE_HALF_H + 0.5
        {
            self.ball.p.x = right_contact;
            let new_speed = (self.current_speed() + BALL_SPEED_INC).min(BALL_SPEED_MAX);
            let offset = ((self.ball.p.y - self.right_y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
            let angle = offset * FRAC_PI_4;

            let lob_pressed = self
                .right_player
                .as_ref()
                .map(|p| p.latest_input.lob_pressed)
                .unwrap_or(false);
            if lob_pressed {
                self.ball.v.z = BALL_LOB_SPEED;
                self.ball.v.x = -(new_speed * BALL_LOB_X_FACTOR) * angle.cos();
            } else {
                self.ball.v.x = -new_speed * angle.cos();
            }
            self.ball.v.y = new_speed * angle.sin();
        }

        // Scoring
        if self.ball.p.x > FIELD_HALF_W {
            self.left_score += 1;
            self.reset_ball(true);
        } else if self.ball.p.x < -FIELD_HALF_W {
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
            ball_pos: [self.ball.p.x, self.ball.p.y, self.ball.p.z],
            ball_vel: [self.ball.v.x, self.ball.v.y, self.ball.v.z],
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
