use std::f32::consts::FRAC_PI_4;
use std::net::UdpSocket;
use std::time::Duration;

use cgmath::{Rotation3, Vector3};
use clap::Parser;
use winit::event::DeviceEvent;
use winit::keyboard::KeyCode;

use etib::camera::{Camera, Projection};
use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::{EngineContext, Game, Scene};

// ---------------------------------------------------------------------------
// Field
// ---------------------------------------------------------------------------
const FIELD_HALF_W: f32 = 40.0;
const FIELD_HALF_H: f32 = 24.0;

// ---------------------------------------------------------------------------
// Paddles
// ---------------------------------------------------------------------------
const PADDLE_X: f32 = 36.0;
/// Half-height of a paddle in cube units (9 cubes → ±4.5).
const PADDLE_HALF_H: f32 = 4.5;
const PADDLE_SPEED: f32 = 24.0;
const SPEED_BOOST_MULTIPLIER: f32 = 2.0;
const SPEED_BOOST_DURATION: f32 = 2.0;
const SPEED_BOOST_COOLDOWN: f32 = 15.0;
/// Maximum paddle center Y so the paddle never clips through a wall.
/// Wall inner face: FIELD_HALF_H − 1.0. Outermost paddle cube face: center + PADDLE_HALF_H + 1.0.
const PADDLE_MAX_Y: f32 = FIELD_HALF_H - PADDLE_HALF_H - 2.0;

// ---------------------------------------------------------------------------
// Ball
// ---------------------------------------------------------------------------
const BALL_SPEED_INIT: f32 = 20.0;
const BALL_SPEED_MAX: f32 = 80.0;
const BALL_SPEED_INC: f32 = 3.0;
const BALL_GRAVITY: f32 = 100.0;
const BALL_LOB_SPEED: f32 = 60.0;
const BALL_LOB_X_FACTOR: f32 = 0.9;
/// Ball center Y limit so it never clips into a wall (wall face − ball radius).
const WALL_LIMIT: f32 = FIELD_HALF_H - 2.0;

// ---------------------------------------------------------------------------
// Camera  — perspective from a low angle to reveal cube depth
// ---------------------------------------------------------------------------
const CAM_EYE: (f32, f32, f32) = (0.0, -60.0, 40.0);
const CAM_TARGET: (f32, f32, f32) = (0.0, 2.0, 0.0);
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
    let mut cubes = Vec::new();
    for y in -4..=4 {
        for x in 0..=1 {
            for z in 0..=1 {
                cubes.push(unit_cube(x as f32, y as f32, z as f32, r, g, b));
            }
        }
    }
    DynamicModel::from_cubes(cubes)
}

fn make_ball() -> DynamicModel {
    let mut cubes = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                cubes.push(unit_cube(
                    x as f32 - 0.5,
                    y as f32 - 0.5,
                    z as f32 - 0.5,
                    1.0,
                    0.95,
                    0.2,
                ));
            }
        }
    }
    DynamicModel::from_cubes(cubes)
}

/// Static geometry: Marseille Orange Velodrome Stadium
fn make_static_geometry() -> Vec<ModelCube> {
    let mut grid: std::collections::HashMap<(i32, i32, i32), ModelCube> =
        std::collections::HashMap::new();

    let mut add_cube = |x: i32, y: i32, z: f32, r: f32, g: f32, b: f32| {
        let key = (x, y, (z * 10.0).round() as i32);
        grid.insert(key, unit_cube(x as f32, y as f32, z, r, g, b));
    };

    let xi = -(FIELD_HALF_W as i32)..=(FIELD_HALF_W as i32);
    for x in xi {
        // Top wall
        add_cube(x, FIELD_HALF_H as i32, 0.0, 0.2, 0.2, 0.2);
        // Bottom wall
        add_cube(x, -(FIELD_HALF_H as i32), 0.0, 0.2, 0.2, 0.2);
    }

    // --- Grass Pitch Floor & Markings ---
    let floor_z = -2.0; // Scaled down to match 2x

    let mut circle_points = std::collections::HashSet::new();
    for angle in 0..72 {
        // Double points
        let rad = angle as f32 * std::f32::consts::PI / 36.0;
        let cx = (rad.cos() * 6.0).round() as i32; // Double radius
        let cy = (rad.sin() * 6.0).round() as i32;
        circle_points.insert((cx, cy));
    }

    let grass_start_x = -(FIELD_HALF_W as i32) - 40;
    let grass_end_x = (FIELD_HALF_W as i32) + 40;
    let grass_start_y = -(FIELD_HALF_H as i32) - 60; // Extend towards the camera
    let grass_end_y = (FIELD_HALF_H as i32) + 40;

    for x in grass_start_x..=grass_end_x {
        for y in grass_start_y..=grass_end_y {
            let is_pitch_x = x >= -(FIELD_HALF_W as i32) && x <= (FIELD_HALF_W as i32);
            let is_pitch_y = y >= -(FIELD_HALF_H as i32) && y <= (FIELD_HALF_H as i32);

            if is_pitch_x && is_pitch_y {
                let is_border_x = x == -(FIELD_HALF_W as i32) || x == FIELD_HALF_W as i32;
                let is_border_y = y == -(FIELD_HALF_H as i32) || y == FIELD_HALF_H as i32;
                let is_center_line = x == 0 || x == -1; // 2 wide
                let is_circle = circle_points.contains(&(x, y));

                let (r, g, b) = if is_border_x || is_border_y || is_center_line || is_circle {
                    (1.0, 1.0, 1.0) // White markings
                } else if (x + FIELD_HALF_W as i32) % 12 >= 6 {
                    (0.1, 0.4, 0.1) // Dark green
                } else {
                    (0.15, 0.45, 0.15) // Light green
                };
                add_cube(x, y, floor_z, r, g, b);
            } else {
                // Outside pitch, just regular grass stripes
                if (x + grass_end_x) % 12 >= 6 {
                    add_cube(x, y, floor_z, 0.1, 0.4, 0.1);
                } else {
                    add_cube(x, y, floor_z, 0.15, 0.45, 0.15);
                }
            }
        }
    }

    // --- Goals Running the Entire Length of the Stadium ---
    let goal_height = 6; // Double height
    for y in -(FIELD_HALF_H as i32)..=(FIELD_HALF_H as i32) {
        // Crossbars
        add_cube(
            -(FIELD_HALF_W as i32) - 1,
            y,
            goal_height as f32,
            1.0,
            1.0,
            1.0,
        );
        add_cube(
            FIELD_HALF_W as i32 + 1,
            y,
            goal_height as f32,
            1.0,
            1.0,
            1.0,
        );

        // Nets (Checkerboard pattern)
        for z in 0..goal_height {
            if (y + z) % 2 == 0 {
                add_cube(-(FIELD_HALF_W as i32) - 2, y, z as f32, 0.5, 0.5, 0.5);
                add_cube(FIELD_HALF_W as i32 + 2, y, z as f32, 0.5, 0.5, 0.5);
            }
        }
    }
    // Side posts
    for z in 0..goal_height {
        add_cube(
            -(FIELD_HALF_W as i32) - 1,
            FIELD_HALF_H as i32,
            z as f32,
            1.0,
            1.0,
            1.0,
        );
        add_cube(
            -(FIELD_HALF_W as i32) - 1,
            -(FIELD_HALF_H as i32),
            z as f32,
            1.0,
            1.0,
            1.0,
        );

        add_cube(
            FIELD_HALF_W as i32 + 1,
            FIELD_HALF_H as i32,
            z as f32,
            1.0,
            1.0,
            1.0,
        );
        add_cube(
            FIELD_HALF_W as i32 + 1,
            -(FIELD_HALF_H as i32),
            z as f32,
            1.0,
            1.0,
            1.0,
        );
    }

    // --- Orange Velodrome Stadium Bleachers ---
    let sky_blue = (0.1, 0.6, 0.9);
    let off_white = (0.85, 0.85, 0.85);
    let text_color = sky_blue;
    let concrete = (0.6, 0.6, 0.6);
    let pure_white = (1.0, 1.0, 1.0);

    let marseille = [
        "111 010 110 111 111 1 100 100 111",
        "101 101 101 100 100 1 100 100 100",
        "101 111 110 111 111 1 100 100 111",
        "101 101 101 001 100 1 100 100 100",
        "101 101 101 111 111 1 111 111 111",
    ];

    let om = ["111 111", "101 101", "101 101", "101 101", "111 101"];

    let year_1899 = [
        "010 111 111 111",
        "110 101 101 101",
        "010 111 111 111",
        "010 101 001 001",
        "111 111 111 111",
    ];

    let max_tiers = 3;
    let rows_per_tier = [16, 24, 32]; // Double rows
    let tier_gaps = [0, 0, 0];

    let mut current_row = 1;
    let mut current_z = 0.0;

    for tier in 0..max_tiers {
        let rows = rows_per_tier[tier];
        let h_offset = tier_gaps[tier];
        current_row += h_offset;

        for r in 1..=rows {
            let out_y_top = FIELD_HALF_H as i32 + current_row;
            let out_x_left = -(FIELD_HALF_W as i32) - current_row;
            let out_x_right = FIELD_HALF_W as i32 + current_row;

            // Keep the slope consistent with doubled rows
            let z_increment = 0.15 + (tier as f32 * 0.05);
            current_z += z_increment;
            let height_z = current_z;

            let ext_x = FIELD_HALF_W as i32 + current_row;
            let ext_y = FIELD_HALF_H as i32 + current_row;

            // Top Sideline (Main Stand)
            for x in -ext_x..=ext_x {
                let mut color = off_white;

                if tier == 1 && r >= 8 && r <= 16 {
                    let text_row = (16 - r) / 2;
                    let start_x = -32;
                    if x >= start_x && x < start_x + 66 {
                        let char_idx = ((x - start_x) / 2) as usize;
                        let line = marseille[text_row as usize].as_bytes();
                        if char_idx < line.len() && line[char_idx] == b'1' {
                            color = text_color;
                        }
                    }
                }

                if (x + ext_x) % 24 == 0 {
                    // Double concrete stairs
                    color = concrete;
                }

                add_cube(x, out_y_top, height_z, color.0, color.1, color.2);
            }

            // Left & Right Goal lines
            let side_start_y = -(FIELD_HALF_H as i32) - 20; // Start further back
            for y in side_start_y..ext_y {
                // LEFT STAND
                let mut color_l = off_white;
                if tier == 1 && r >= 8 && r <= 16 {
                    let text_row = (16 - r) / 2;
                    let start_y = -6;
                    if y >= start_y && y < start_y + 14 {
                        let char_idx = ((y - start_y) / 2) as usize;
                        let line = om[text_row as usize].as_bytes();
                        if char_idx < line.len() && line[char_idx] == b'1' {
                            color_l = text_color;
                        }
                    }
                }
                if (y - side_start_y) % 24 == 0 {
                    color_l = concrete;
                }
                add_cube(out_x_left, y, height_z, color_l.0, color_l.1, color_l.2);

                // RIGHT STAND
                let mut color_r = off_white;
                if tier == 1 && r >= 8 && r <= 16 {
                    let text_row = (16 - r) / 2;
                    let start_y = 14;
                    if y <= start_y && y > start_y - 30 {
                        let char_idx = ((start_y - y) / 2) as usize;
                        let line = year_1899[text_row as usize].as_bytes();
                        if char_idx < line.len() && line[char_idx] == b'1' {
                            color_r = text_color;
                        }
                    }
                }
                if (y - side_start_y) % 24 == 0 {
                    color_r = concrete;
                }
                add_cube(out_x_right, y, height_z, color_r.0, color_r.1, color_r.2);
            }

            // Iconic Velodrome undulating roof smoothly connecting to the stands
            if tier == 2 && r == rows {
                let overhang_max = 40;
                for overhang in 1..=overhang_max {
                    let roof_y = out_y_top - overhang;
                    let wave_amplitude = 6.0 * (overhang as f32 / overhang_max as f32);
                    for x in -ext_x..=ext_x {
                        let wave = (x as f32 * 0.075).sin() * wave_amplitude; // Halve frequency, double amplitude
                        let roof_z = height_z + (overhang as f32 * 0.2) + wave;
                        add_cube(x, roof_y, roof_z, pure_white.0, pure_white.1, pure_white.2);
                    }
                }

                for overhang in 1..=overhang_max {
                    let wave_amplitude = 6.0 * (overhang as f32 / overhang_max as f32);
                    let roof_limit_y = out_y_top - overhang_max;
                    for y in side_start_y..roof_limit_y {
                        let roof_x_l = out_x_left + overhang;
                        let wave_l = (y as f32 * 0.075).cos() * wave_amplitude;
                        let roof_z_l = height_z + (overhang as f32 * 0.2) + wave_l;
                        add_cube(
                            roof_x_l,
                            y,
                            roof_z_l,
                            pure_white.0,
                            pure_white.1,
                            pure_white.2,
                        );

                        let roof_x_r = out_x_right - overhang;
                        let wave_r = (y as f32 * 0.075).cos() * wave_amplitude;
                        let roof_z_r = height_z + (overhang as f32 * 0.2) + wave_r;
                        add_cube(
                            roof_x_r,
                            y,
                            roof_z_r,
                            pure_white.0,
                            pure_white.1,
                            pure_white.2,
                        );
                    }
                }
            }

            current_row += 1;
        }
    }

    grid.into_values().collect()
}

// ---------------------------------------------------------------------------
// 3D Score Rendering
// ---------------------------------------------------------------------------

fn make_char_cubes(c: char, offset_x: f32, color: Vector3<f32>) -> Vec<ModelCube> {
    let mut cubes = Vec::new();
    let map = match c.to_ascii_uppercase() {
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
        'P' => [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0],
        'O' => [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1],
        'N' => [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1],
        'G' => [1, 1, 1, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 1, 1],
        'D' => [1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0],
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
        cubes.extend(make_char_cubes(c, offset_x, color));
        offset_x += 4.0; // Spacing between digits
    }
    let mut model = DynamicModel::from_cubes(cubes);
    // Float the score board in the air
    model.position = Vector3::new(0.0, FIELD_HALF_H, 10.0);
    model
}

fn make_text_model(text: &str, base_x: f32, color: Vector3<f32>) -> DynamicModel {
    let mut cubes = Vec::new();
    let mut offset_x = base_x;
    for c in text.chars() {
        cubes.extend(make_char_cubes(c, offset_x, color));
        offset_x += 4.0;
    }
    DynamicModel::from_cubes(cubes)
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------
struct PongGame {
    scene: Scene,
    ball: Ball,
    left_player: Player,
    right_player: Player,

    game_started: bool,
    paused: bool,
    prev_start_pressed: bool,

    // Networking
    client_socket: Option<UdpSocket>,
    server_addr: Option<String>,
    seq_num: u32,

    // Trail
    trail_ids: Vec<usize>,
    trail_idx: usize,
    last_trail_pos: (i32, i32),
    shadow_id: usize,

    // Particles
    particles: Vec<Particle>,

    // Animation time
    time: f32,

    title_id: usize,
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

struct Particle {
    id: usize,
    velocity: Vector3<f32>,
    life: f32,
}

fn spawn_explosion(
    scene: &mut Scene,
    particles: &mut Vec<Particle>,
    origin: Vector3<f32>,
    color: Vector3<f32>,
) {
    let num_particles = 60;
    for i in 0..num_particles {
        let mut model =
            DynamicModel::from_cubes(vec![unit_cube(-0.5, -0.5, -0.5, color.x, color.y, color.z)]);
        model.position = origin;
        model.position.x += (i as f32 * 1.3).sin() * 2.0;
        model.position.y += (i as f32 * 1.7).cos() * 2.0;
        model.position.z += (i as f32 * 2.3).sin() * 2.0;

        let id = scene.add_dynamic(model);

        let angle1 = i as f32 * std::f32::consts::PI * 2.0 / num_particles as f32;
        let angle2 = (i as f32 * 13.0).sin() * std::f32::consts::FRAC_PI_4;
        let speed = 40.0 + (i as f32 * 7.0).sin().abs() * 60.0;

        let vx = angle1.cos() * angle2.cos() * speed;
        let vy = angle1.sin() * angle2.cos() * speed;
        let vz = angle2.sin().abs() * speed + 20.0;

        particles.push(Particle {
            id,
            velocity: Vector3::new(vx, vy, vz),
            life: 1.0 + (i as f32 * 5.0).sin().abs() * 1.0,
        });
    }
}

// ---------------------------------------------------------------------------
// Shared physics — used by both solo and server
// ---------------------------------------------------------------------------

fn ball_speed(ball: &Ball) -> f32 {
    (ball.v.x.powi(2) + ball.v.y.powi(2)).sqrt()
}

fn reset_ball(ball: &mut Ball, toward_right: bool) {
    ball.p = Vector3::new(0.0, 0.0, 0.0);
    let vx = if toward_right {
        BALL_SPEED_INIT
    } else {
        -BALL_SPEED_INIT
    };
    ball.v = Vector3::new(vx, BALL_SPEED_INIT * 0.4, 0.0);
}

fn step_physics(
    ball: &mut Ball,
    left: &mut Player,
    right: &mut Player,
    left_input: &PlayerInput,
    right_input: &PlayerInput,
    dt: f32,
) {
    // --- Boost timers ---
    if left.cooldown_timer > 0.0 {
        left.cooldown_timer -= dt;
    }
    if left.boost_timer > 0.0 {
        left.boost_timer -= dt;
        if left.boost_timer <= 0.0 {
            left.boost_active = false;
        }
    }
    if right.cooldown_timer > 0.0 {
        right.cooldown_timer -= dt;
    }
    if right.boost_timer > 0.0 {
        right.boost_timer -= dt;
        if right.boost_timer <= 0.0 {
            right.boost_active = false;
        }
    }

    // --- Boost activation ---
    if left_input.boost_pressed && left.cooldown_timer <= 0.0 {
        left.boost_active = true;
        left.boost_timer = SPEED_BOOST_DURATION;
        left.cooldown_timer = SPEED_BOOST_COOLDOWN;
    }
    if right_input.boost_pressed && right.cooldown_timer <= 0.0 {
        right.boost_active = true;
        right.boost_timer = SPEED_BOOST_DURATION;
        right.cooldown_timer = SPEED_BOOST_COOLDOWN;
    }

    let left_speed = if left.boost_active {
        PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
    } else {
        PADDLE_SPEED
    };
    let right_speed = if right.boost_active {
        PADDLE_SPEED * SPEED_BOOST_MULTIPLIER
    } else {
        PADDLE_SPEED
    };

    // --- Paddle movement ---
    let mut left_mx = left_input.move_x;
    let mut left_my = left_input.move_y;
    let left_mag = (left_mx * left_mx + left_my * left_my).sqrt();
    if left_mag > 1.0 {
        left_mx /= left_mag;
        left_my /= left_mag;
    }

    left.x = (left.x + left_mx * left_speed * dt).clamp(-FIELD_HALF_W + 0.5, -2.0); // Don't cross centre
    left.y = (left.y + left_my * left_speed * dt).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);

    let mut right_mx = right_input.move_x;
    let mut right_my = right_input.move_y;
    let right_mag = (right_mx * right_mx + right_my * right_my).sqrt();
    if right_mag > 1.0 {
        right_mx /= right_mag;
        right_my /= right_mag;
    }

    right.x = (right.x + right_mx * right_speed * dt).clamp(2.0, FIELD_HALF_W - 1.5); // Don't cross centre
    right.y = (right.y + right_my * right_speed * dt).clamp(-PADDLE_MAX_Y, PADDLE_MAX_Y);

    // --- Ball movement ---
    ball.p += ball.v * dt;
    ball.v.z -= BALL_GRAVITY * dt;

    // Floor bounce / clamp
    if ball.p.z <= 0.0 {
        ball.p.z = 0.0;
        if ball.v.z < -5.0 {
            ball.v.z *= -0.5;
        } else {
            ball.v.z = 0.0;
        }
    }

    // --- Top / bottom wall bounce ---
    if ball.p.y > WALL_LIMIT {
        ball.p.y = WALL_LIMIT;
        ball.v.y = -ball.v.y.abs();
    } else if ball.p.y < -WALL_LIMIT {
        ball.p.y = -WALL_LIMIT;
        ball.v.y = ball.v.y.abs();
    }

    // --- Paddle collisions ---
    // Ball can only hit paddles if it is low enough (e.g. not lobbed over them)
    let ball_is_hittable = ball.p.z < 3.0;

    let left_contact = left.x + 2.5;
    if ball_is_hittable
        && ball.v.x < 0.0
        && ball.p.x <= left_contact
        && ball.p.x > left.x + 0.0 // prevent hitting with back of paddle
        && (ball.p.y - left.y).abs() <= PADDLE_HALF_H
    // prevent hitting with sides
    {
        ball.p.x = left_contact;
        let new_speed = (ball_speed(ball) + BALL_SPEED_INC).min(BALL_SPEED_MAX);
        let offset = ((ball.p.y - left.y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
        let angle = offset * FRAC_PI_4;
        if left_input.lob_pressed {
            ball.v.z = BALL_LOB_SPEED;
            ball.v.x = (new_speed * BALL_LOB_X_FACTOR) * angle.cos();
        } else {
            ball.v.x = new_speed * angle.cos();
        }
        ball.v.y = new_speed * angle.sin();
    }

    let right_contact = right.x - 1.5;
    if ball_is_hittable
        && ball.v.x > 0.0
        && ball.p.x >= right_contact
        && ball.p.x < right.x + 1.0 // prevent hitting with back of paddle
        && (ball.p.y - right.y).abs() <= PADDLE_HALF_H
    // prevent hitting with sides
    {
        ball.p.x = right_contact;
        let new_speed = (ball_speed(ball) + BALL_SPEED_INC).min(BALL_SPEED_MAX);
        let offset = ((ball.p.y - right.y) / PADDLE_HALF_H).clamp(-1.0, 1.0);
        let angle = offset * FRAC_PI_4;
        if right_input.lob_pressed {
            ball.v.z = BALL_LOB_SPEED;
            ball.v.x = -(new_speed * BALL_LOB_X_FACTOR) * angle.cos();
        } else {
            ball.v.x = -new_speed * angle.cos();
        }
        ball.v.y = new_speed * angle.sin();
    }

    // --- Scoring ---
    if ball.p.x > FIELD_HALF_W {
        left.score += 1;
        reset_ball(ball, true); // serve toward right (they just missed)
    } else if ball.p.x < -FIELD_HALF_W {
        right.score += 1;
        reset_ball(ball, false); // serve toward left (they just missed)
    }
}

impl Game for PongGame {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _params: ()) -> Self {
        ctx.set_window_title("Pong 3D");

        let aspect = ctx.window_size().width as f32 / ctx.window_size().height as f32;
        let camera = Camera::new(
            ctx,
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

        let max_dyn = left_paddle.cube_count()
            + right_paddle.cube_count()
            + ball_model.cube_count()
            + 100
            + 2000; // room for score numbers and particles

        let mut scene = Scene::new(ctx, camera, &walls, max_dyn);

        let gfx = &ctx.gfx;
        let device = gfx.device();
        scene
            .set_skybox_from_bytes(
                device,
                &gfx.queue,
                include_bytes!("sky.hdr"),
                1080,
                cgmath::Matrix4::from_angle_x(cgmath::Deg(-75.0)),
            )
            .expect("Failed to load skybox");

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

        // Hide models initially for the startup scene
        scene.get_dynamic_mut(left_player.id).unwrap().position = Vector3::new(0.0, 0.0, 1000.0);
        scene.get_dynamic_mut(right_player.id).unwrap().position = Vector3::new(0.0, 0.0, 1000.0);
        scene.get_dynamic_mut(ball.id).unwrap().position = Vector3::new(0.0, 0.0, 1000.0);
        if let Some(id) = left_player.score_id {
            scene.get_dynamic_mut(id).unwrap().position = Vector3::new(0.0, 0.0, 1000.0);
        }
        if let Some(id) = right_player.score_id {
            scene.get_dynamic_mut(id).unwrap().position = Vector3::new(0.0, 0.0, 1000.0);
        }

        let mut trail_ids = Vec::new();
        for _ in 0..10 {
            let mut trail_cube =
                DynamicModel::from_cubes(vec![unit_cube(0.0, 0.0, 0.0, 5.0, 0.0, 0.0)]);
            trail_cube.position = Vector3::new(0.0, 0.0, 1000.0);
            trail_ids.push(scene.add_dynamic(trail_cube));
        }

        let mut shadow_cubes = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                shadow_cubes.push(unit_cube(
                    x as f32 - 0.5,
                    y as f32 - 0.5,
                    0.0,
                    0.05,
                    0.05,
                    0.05,
                ));
            }
        }
        let mut shadow_model = DynamicModel::from_cubes(shadow_cubes);
        shadow_model.position = Vector3::new(0.0, 0.0, 1000.0);
        let shadow_id = scene.add_dynamic(shadow_model);

        let title_model = make_text_model("PONG 3D", -14.0, Vector3::new(0.0, 0.8, 1.0));
        let mut title_dynamic = title_model;
        title_dynamic.position = Vector3::new(0.0, 0.0, 15.0); // Center of field, floating
        let title_id = scene.add_dynamic(title_dynamic);

        PongGame {
            scene,
            ball,
            left_player,
            right_player,
            game_started: false,
            paused: false,
            prev_start_pressed: false,
            client_socket,
            server_addr,
            seq_num: 0,
            trail_ids,
            trail_idx: 0,
            last_trail_pos: (0, 0),
            shadow_id,
            particles: Vec::new(),
            time: 0.0,
            title_id,
        }
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.time.dt;
        self.time += dt;

        if self.game_started {
            if let Some(m) = self.scene.get_dynamic_mut(self.title_id) {
                m.position.z = 1000.0;
            }
        } else {
            // Floating animation for the title
            if let Some(m) = self.scene.get_dynamic_mut(self.title_id) {
                m.position.z = 15.0 + (self.time * 2.0).sin() * 2.0;
                m.rotation =
                    cgmath::Quaternion::from_angle_z(cgmath::Deg((self.time * 20.0).sin() * 5.0));
            }
        }

        if !self.game_started {
            return;
        }

        let mut start_pressed = false;
        for (_, gamepad) in ctx.gilrs.gamepads() {
            if gamepad.is_pressed(gilrs::Button::Start) {
                start_pressed = true;
            }
        }
        let start_just_pressed = start_pressed && !self.prev_start_pressed;
        self.prev_start_pressed = start_pressed;

        if ctx.input.is_key_just_pressed(KeyCode::Escape) || start_just_pressed {
            self.paused = !self.paused;
        }

        if self.paused {
            return;
        }

        let prev_ball_pos = self.ball.p;

        // Apply local physics if NOT connected to a server
        if self.client_socket.is_none() {
            let left_input = PlayerInput::left_input(ctx);
            let right_input = PlayerInput::right_input(ctx);

            step_physics(
                &mut self.ball,
                &mut self.left_player,
                &mut self.right_player,
                &left_input,
                &right_input,
                dt,
            );
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

            spawn_explosion(
                scene,
                &mut self.particles,
                Vector3::new(FIELD_HALF_W, prev_ball_pos.y, prev_ball_pos.z),
                left_color,
            );
        }

        if self.right_player.score as i32 != self.right_player.rendered_score {
            if let Some(old_id) = self.right_player.score_id {
                scene.remove_dynamic(old_id);
            }
            let right_color = Vector3::new(1.0, 0.45, 0.1);
            let right_score_model = make_score_model(self.right_player.score, 10.0, right_color);
            self.right_player.score_id = Some(scene.add_dynamic(right_score_model));
            self.right_player.rendered_score = self.right_player.score as i32;

            spawn_explosion(
                scene,
                &mut self.particles,
                Vector3::new(-FIELD_HALF_W, prev_ball_pos.y, prev_ball_pos.z),
                right_color,
            );
        }

        // --- Sync GPU transforms ---
        scene.get_dynamic_mut(self.ball.id).unwrap().position =
            Vector3::new(self.ball.p.x, self.ball.p.y, self.ball.p.z);

        let now = self.time;
        let left_color = if self.left_player.boost_active {
            let r = (now * 15.0).sin() * 0.5 + 0.5;
            let g = (now * 15.0 + 2.094).sin() * 0.5 + 0.5;
            let b = (now * 15.0 + 4.188).sin() * 0.5 + 0.5;
            Vector3::new(r, g, b)
        } else {
            Vector3::new(0.2, 0.75, 1.0)
        };

        let right_color = if self.right_player.boost_active {
            let r = (now * 15.0).sin() * 0.5 + 0.5;
            let g = (now * 15.0 + 2.094).sin() * 0.5 + 0.5;
            let b = (now * 15.0 + 4.188).sin() * 0.5 + 0.5;
            Vector3::new(r, g, b)
        } else {
            Vector3::new(1.0, 0.45, 0.1)
        };

        if let Some(left_model) = scene.get_dynamic_mut(self.left_player.id) {
            left_model.position = Vector3::new(self.left_player.x, self.left_player.y, 0.0);
            for cube in left_model.cubes_mut() {
                cube.color = left_color;
            }
        }
        if let Some(right_model) = scene.get_dynamic_mut(self.right_player.id) {
            right_model.position = Vector3::new(self.right_player.x, self.right_player.y, 0.0);
            for cube in right_model.cubes_mut() {
                cube.color = right_color;
            }
        }

        // --- Trail Update ---
        let ball_grid_x = self.ball.p.x.round() as i32;
        let ball_grid_y = self.ball.p.y.round() as i32;

        if self.ball.p.z > 0.1 {
            // Hide trail
            for &trail_id in &self.trail_ids {
                if let Some(model) = scene.get_dynamic_mut(trail_id) {
                    model.position = Vector3::new(0.0, 0.0, 1000.0);
                }
            }
            // Show shadow
            if let Some(shadow_model) = scene.get_dynamic_mut(self.shadow_id) {
                shadow_model.position = Vector3::new(self.ball.p.x, self.ball.p.y, -1.95);
            }
            // Keep last_trail_pos updated
            self.last_trail_pos = (ball_grid_x, ball_grid_y);
        } else {
            // Hide shadow
            if let Some(shadow_model) = scene.get_dynamic_mut(self.shadow_id) {
                shadow_model.position = Vector3::new(0.0, 0.0, 1000.0);
            }

            if (ball_grid_x, ball_grid_y) != self.last_trail_pos {
                self.last_trail_pos = (ball_grid_x, ball_grid_y);
                let trail_id = self.trail_ids[self.trail_idx];
                if let Some(model) = scene.get_dynamic_mut(trail_id) {
                    model.position = Vector3::new(ball_grid_x as f32, ball_grid_y as f32, -1.95);
                }
                self.trail_idx = (self.trail_idx + 1) % 10;
            }
        }

        // --- Particles Update ---
        let mut i = 0;
        while i < self.particles.len() {
            self.particles[i].life -= dt;
            if self.particles[i].life <= 0.0 {
                scene.remove_dynamic(self.particles[i].id);
                self.particles.swap_remove(i);
            } else {
                let p = &mut self.particles[i];
                p.velocity.z -= BALL_GRAVITY * 1.5 * dt; // gravity
                if let Some(model) = scene.get_dynamic_mut(p.id) {
                    model.position += p.velocity * dt;

                    // floor bounce
                    if model.position.z < 0.0 {
                        model.position.z = 0.0;
                        p.velocity.z *= -0.5;
                        p.velocity.x *= 0.8;
                        p.velocity.y *= 0.8;
                    }

                    // optional: dim color over time
                    if let Some(cube) = model.cubes_mut().first_mut() {
                        cube.color *= 0.95;
                    }
                }
                i += 1;
            }
        }

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

            let mut lob_pressed = ctx.input.is_key_pressed(KeyCode::ShiftLeft)
                || ctx.input.is_key_pressed(KeyCode::ShiftRight);
            let mut boost_pressed = ctx.input.is_key_just_pressed(KeyCode::ControlLeft)
                || ctx.input.is_key_just_pressed(KeyCode::Enter);

            // Gamepad input for client
            if let Some((gx, gy, glob, gboost)) = PlayerInput::gamepad_input(ctx, 0) {
                if gy != 0.0 {
                    move_y = gy;
                }
                if gx != 0.0 {
                    move_x = gx;
                }
                lob_pressed |= glob;
                boost_pressed |= gboost;
            }

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
        if !self.game_started {
            etib::egui::Area::new(etib::egui::Id::new("Main Menu"))
                .anchor(etib::egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);

                        let btn_size = etib::egui::vec2(250.0, 60.0);
                        if ui
                            .add_sized(
                                btn_size,
                                etib::egui::Button::new(
                                    etib::egui::RichText::new("START GAME").size(28.0),
                                ),
                            )
                            .clicked()
                        {
                            self.game_started = true;

                            // Restore score positions
                            if let Some(id) = self.left_player.score_id {
                                if let Some(model) = self.scene.get_dynamic_mut(id) {
                                    model.position = Vector3::new(0.0, FIELD_HALF_H, 10.0);
                                }
                            }
                            if let Some(id) = self.right_player.score_id {
                                if let Some(model) = self.scene.get_dynamic_mut(id) {
                                    model.position = Vector3::new(0.0, FIELD_HALF_H, 10.0);
                                }
                            }
                        }
                        ui.add_space(15.0);
                        if ui
                            .add_sized(
                                btn_size,
                                etib::egui::Button::new(
                                    etib::egui::RichText::new("EXIT").size(28.0),
                                ),
                            )
                            .clicked()
                        {
                            std::process::exit(0);
                        }
                        ui.add_space(20.0);
                    });
                });
            return;
        }

        if self.paused {
            etib::egui::Window::new("Pause Menu")
                .anchor(etib::egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .frame(etib::egui::Frame::NONE.fill(etib::egui::Color32::from_black_alpha(100)))
                .show(ui_ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            etib::egui::RichText::new("PAUSED")
                                .size(40.0)
                                .strong()
                                .color(etib::egui::Color32::from_rgb(255, 180, 50)),
                        );
                        ui.add_space(30.0);

                        let btn_size = etib::egui::vec2(220.0, 50.0);
                        if ui
                            .add_sized(
                                btn_size,
                                etib::egui::Button::new(
                                    etib::egui::RichText::new("RESUME").size(24.0),
                                ),
                            )
                            .clicked()
                        {
                            self.paused = false;
                        }
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                btn_size,
                                etib::egui::Button::new(
                                    etib::egui::RichText::new("RESTART").size(24.0),
                                ),
                            )
                            .clicked()
                        {
                            self.paused = false;
                            self.left_player.score = 0;
                            self.right_player.score = 0;
                            self.left_player.y = 0.0;
                            self.right_player.y = 0.0;
                            reset_ball(&mut self.ball, true);
                        }
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                btn_size,
                                etib::egui::Button::new(
                                    etib::egui::RichText::new("QUIT").size(24.0),
                                ),
                            )
                            .clicked()
                        {
                            std::process::exit(0);
                        }
                        ui.add_space(20.0);
                    });
                });
            return;
        }

        // etib::egui::Area::new(etib::egui::Id::new("pong_debug_info")).show(ui_ctx, |ui| {
        //     ui.with_layout(
        //         etib::egui::Layout::left_to_right(etib::egui::Align::Center),
        //         |ui| {
        //             if ui.button("Randomize Ball").clicked() {
        //                 let now = ctx.now();
        //                 let r = (now * 13.0).sin().abs() as f32;
        //                 let g = (now * 17.0).sin().abs() as f32;
        //                 let b = (now * 19.0).sin().abs() as f32;

        //                 let scene = &mut self.scene;
        //                 if let Some(ball) = scene.get_dynamic_mut(self.ball.id) {
        //                     for cube in ball.cubes_mut() {
        //                         cube.color = cgmath::Vector3::new(r, g, b);
        //                     }
        //                 }
        //             }

        //             ui.separator();
        //             ui.heading("Boosts:");
        //             ui.label(format!(
        //                 "Left Cooldown: {:.1}s",
        //                 self.left_player.cooldown_timer.max(0.0)
        //             ));
        //             ui.label(format!(
        //                 "Right Cooldown: {:.1}s",
        //                 self.right_player.cooldown_timer.max(0.0)
        //             ));

        //             ui.separator();
        //             ui.heading("Ball Info:");
        //             ui.label(format!("Speed X: {:.2}", self.ball.v.x));
        //             ui.label(format!("Speed Y: {:.2}", self.ball.v.y));

        //             ui.separator();
        //             ui.label(format!("FPS: {:.0}", ctx.fps()));
        //         },
        //     );
        // });
    }

    fn scene(&mut self) -> Option<&mut etib::Scene> {
        Some(&mut self.scene)
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

impl PlayerInput {
    pub fn gamepad_input(ctx: &EngineContext, player_idx: usize) -> Option<(f32, f32, bool, bool)> {
        let mut gamepads = ctx.gilrs.gamepads();
        let (_, gamepad) = gamepads.nth(player_idx)?;

        let mut move_y = gamepad.value(gilrs::Axis::LeftStickY);
        let mut move_x = gamepad.value(gilrs::Axis::LeftStickX);

        if gamepad.is_pressed(gilrs::Button::DPadUp) {
            move_y = 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadDown) {
            move_y = -1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadRight) {
            move_x = 1.0;
        }
        if gamepad.is_pressed(gilrs::Button::DPadLeft) {
            move_x = -1.0;
        }

        // Deadzone check
        if move_y.abs() < 0.1 {
            move_y = 0.0;
        }
        if move_x.abs() < 0.1 {
            move_x = 0.0;
        }

        let lob = gamepad.is_pressed(gilrs::Button::RightTrigger2)
            || gamepad.is_pressed(gilrs::Button::RightTrigger)
            || gamepad.is_pressed(gilrs::Button::East);
        let boost = gamepad.is_pressed(gilrs::Button::South)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger)
            || gamepad.is_pressed(gilrs::Button::LeftTrigger2);

        Some((move_x, move_y, lob, boost))
    }

    fn left_input(ctx: &EngineContext) -> Self {
        let (mut move_x, mut move_y, mut lob_pressed, mut boost_pressed) =
            Self::gamepad_input(ctx, 0).unwrap_or((0.0, 0.0, false, false));

        if ctx.input.is_key_pressed(KeyCode::KeyW) {
            move_y = 1.0;
        } else if ctx.input.is_key_pressed(KeyCode::KeyS) {
            move_y = -1.0;
        }

        if ctx.input.is_key_pressed(KeyCode::KeyD) {
            move_x = 1.0;
        } else if ctx.input.is_key_pressed(KeyCode::KeyA) {
            move_x = -1.0;
        }

        lob_pressed |= ctx.input.is_key_pressed(KeyCode::ShiftLeft);
        boost_pressed |= ctx.input.is_key_just_pressed(KeyCode::ControlLeft);

        PlayerInput {
            move_y,
            move_x,
            lob_pressed,
            boost_pressed,
            sequence_number: 0,
        }
    }

    fn right_input(ctx: &EngineContext) -> Self {
        let (mut move_x, mut move_y, mut lob_pressed, mut boost_pressed) =
            Self::gamepad_input(ctx, 1).unwrap_or((0.0, 0.0, false, false));

        if ctx.input.is_key_pressed(KeyCode::ArrowUp) {
            move_y = 1.0;
        } else if ctx.input.is_key_pressed(KeyCode::ArrowDown) {
            move_y = -1.0;
        }

        if ctx.input.is_key_pressed(KeyCode::ArrowRight) {
            move_x = 1.0;
        } else if ctx.input.is_key_pressed(KeyCode::ArrowLeft) {
            move_x = -1.0;
        }

        lob_pressed |= ctx.input.is_key_pressed(KeyCode::ShiftRight);
        boost_pressed |= ctx.input.is_key_just_pressed(KeyCode::Enter);

        PlayerInput {
            move_y,
            move_x,
            lob_pressed,
            boost_pressed,
            sequence_number: 0,
        }
    }
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
}

struct PongServer {
    socket: UdpSocket,

    // Physics State
    ball: Ball,
    left_player: Player,
    right_player: Player,

    // Networking State
    left_net: Option<ServerPlayerState>,
    right_net: Option<ServerPlayerState>,
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
            left_player: Player {
                x: -PADDLE_X,
                y: 0.0,
                boost_active: false,
                boost_timer: 0.0,
                cooldown_timer: 0.0,
                score: 0,
                rendered_score: 0,
                id: 0,
                score_id: None,
            },
            right_player: Player {
                x: PADDLE_X,
                y: 0.0,
                boost_active: false,
                boost_timer: 0.0,
                cooldown_timer: 0.0,
                score: 0,
                rendered_score: 0,
                id: 0,
                score_id: None,
            },
            left_net: None,
            right_net: None,
        })
    }

    fn receive_inputs(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 1024];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, src)) => {
                    if let Ok(input) = bincode::deserialize::<PlayerInput>(&buf[..size]) {
                        // Assignment
                        let is_left = self.left_net.as_ref().map_or(false, |p| p.addr == src);
                        let is_right = self.right_net.as_ref().map_or(false, |p| p.addr == src);

                        if !is_left && !is_right {
                            if self.left_net.is_none() {
                                log::info!("Left player registered: {}", src);
                                self.left_net = Some(ServerPlayerState {
                                    addr: src,
                                    latest_input: input.clone(),
                                    last_seq_processed: input.sequence_number,
                                });
                            } else if self.right_net.is_none() {
                                log::info!("Right player registered: {}", src);
                                self.right_net = Some(ServerPlayerState {
                                    addr: src,
                                    latest_input: input.clone(),
                                    last_seq_processed: input.sequence_number,
                                });
                            }
                        } else if is_left {
                            if let Some(p) = &mut self.left_net {
                                if input.sequence_number > p.last_seq_processed {
                                    p.latest_input = input;
                                }
                            }
                        } else if is_right {
                            if let Some(p) = &mut self.right_net {
                                if input.sequence_number > p.last_seq_processed {
                                    p.latest_input = input;
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
        let left_input = self
            .left_net
            .as_ref()
            .map(|p| p.latest_input.clone())
            .unwrap_or_default();
        let right_input = self
            .right_net
            .as_ref()
            .map(|p| p.latest_input.clone())
            .unwrap_or_default();

        step_physics(
            &mut self.ball,
            &mut self.left_player,
            &mut self.right_player,
            &left_input,
            &right_input,
            dt,
        );

        // Update sequence numbers
        if let Some(p) = &mut self.left_net {
            p.last_seq_processed = p.latest_input.sequence_number;
        }
        if let Some(p) = &mut self.right_net {
            p.last_seq_processed = p.latest_input.sequence_number;
        }
    }

    fn broadcast_state(&mut self) -> anyhow::Result<()> {
        let snap = GameStateSnapshot {
            ball_pos: [self.ball.p.x, self.ball.p.y, self.ball.p.z],
            ball_vel: [self.ball.v.x, self.ball.v.y, self.ball.v.z],
            left_pos: [self.left_player.x, self.left_player.y],
            left_boost_active: self.left_player.boost_active,
            right_pos: [self.right_player.x, self.right_player.y],
            right_boost_active: self.right_player.boost_active,
            left_score: self.left_player.score,
            right_score: self.right_player.score,
            auth_sequence_left: self
                .left_net
                .as_ref()
                .map(|p| p.last_seq_processed)
                .unwrap_or(0),
            auth_sequence_right: self
                .right_net
                .as_ref()
                .map(|p| p.last_seq_processed)
                .unwrap_or(0),
        };

        let bytes = bincode::serialize(&snap)?;

        if let Some(p) = &self.left_net {
            let _ = self.socket.send_to(&bytes, p.addr);
        }
        if let Some(p) = &self.right_net {
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
