/// Physics demo — rigid-body cubes with gravity, angular momentum, wall bounce,
/// and cube-to-cube impulse collision.
///
/// Controls:
///   C      — grab cursor / free-look
///   Escape — release cursor
///   WASD   — fly camera (while cursor grabbed)
///   R      — restart (randomize spawns)
use cgmath::{InnerSpace, Vector3};
use winit::event::DeviceEvent;
use winit::keyboard::{KeyCode, PhysicalKey};

use etib::camera::{Camera, CameraController, Projection};
use etib::config::EngineConfig;
use etib::cube::{DynamicModel, ModelCube};
use etib::physics::{self, RigidBody};
use etib::{EngineContext, Game, Scene};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GROUND_HALF: i32 = 15;

/// Y of the floor surface (cubes centred at y=0 → top face at +0.5).
const FLOOR_TOP: f32 = 0.5;

/// Half-side of each cube model (2×2×2 block → 1 unit centre-to-face).
const CUBE_HALF: f32 = 1.0;

/// Physics wall bound.  Cube corners are kept strictly inside ±WALL on X and Z.
/// Set equal to GROUND_HALF−1.5 so the outermost floor tiles act as the wall face.
const WALL: f32 = (GROUND_HALF as f32) - 1.5;

const CUBE_COUNT: usize = 8;

const COLORS: [(f32, f32, f32); CUBE_COUNT] = [
    (1.0, 0.20, 0.20),
    (1.0, 0.55, 0.05),
    (1.0, 0.90, 0.10),
    (0.20, 1.0, 0.20),
    (0.10, 0.80, 1.0),
    (0.20, 0.40, 1.0),
    (0.75, 0.20, 1.0),
    (1.0, 1.0, 1.0),
];

// ---------------------------------------------------------------------------
// Scene geometry helpers
// ---------------------------------------------------------------------------

fn make_floor() -> Vec<ModelCube> {
    let mut cubes = Vec::new();
    for x in -GROUND_HALF..GROUND_HALF {
        for z in -GROUND_HALF..GROUND_HALF {
            let dx = (x + GROUND_HALF).min(GROUND_HALF - 1 - x);
            let dz = (z + GROUND_HALF).min(GROUND_HALF - 1 - z);
            let edge = dx.min(dz);

            let gray = if edge == 0 {
                // Outermost ring — very dark wall indicator
                0.06
            } else if edge == 1 {
                // Second ring — darker shadow
                0.14
            } else {
                let bright = (x + z).rem_euclid(2) == 0;
                if bright { 0.65 } else { 0.30 }
            };
            cubes.push(ModelCube {
                position: Vector3::new(x as f32, 0.0, z as f32),
                color: Vector3::new(gray, gray, gray),
            });
        }
    }
    cubes
}

/// Thin visible wall panels — one cube tall, placed just outside the play area.
/// Coloured a muted blue-grey so they read as transparent-ish barriers.
fn make_wall_panels() -> Vec<ModelCube> {
    let mut cubes = Vec::new();
    let half = GROUND_HALF;
    let color = Vector3::new(0.18, 0.22, 0.40);

    // North / south walls (full width)
    for x in -half..half {
        for &z in &[-(half as f32), (half - 1) as f32] {
            cubes.push(ModelCube {
                position: Vector3::new(x as f32, 1.0, z),
                color,
            });
        }
    }
    // East / west walls (skip corners already covered above)
    for z in -(half - 1)..half - 1 {
        for &x in &[-(half as f32), (half - 1) as f32] {
            cubes.push(ModelCube {
                position: Vector3::new(x, 1.0, z as f32),
                color,
            });
        }
    }

    cubes
}

fn make_cube_model(r: f32, g: f32, b: f32) -> DynamicModel {
    let mut voxels = Vec::new();
    for xi in 0..2i32 {
        for yi in 0..2i32 {
            for zi in 0..2i32 {
                voxels.push(ModelCube {
                    position: Vector3::new(xi as f32 - 0.5, yi as f32 - 0.5, zi as f32 - 0.5),
                    color: Vector3::new(r, g, b),
                });
            }
        }
    }
    DynamicModel::from_cubes(voxels)
}

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

struct Cube {
    id: usize,
    body: RigidBody,
}

struct PhysicsDemo {
    scene: Scene,
    camera_controller: CameraController,
    cursor_grabbed: bool,
    cubes: Vec<Cube>,
    rng_state: u64,
}

impl Game for PhysicsDemo {
    type InitParams = ();

    fn init(ctx: &mut EngineContext, _: ()) -> Self {
        let size = ctx.window_size();
        let aspect = size.width as f32 / size.height as f32;

        let eye = cgmath::Point3::new(0.0_f32, 30.0, 45.0);
        let target = cgmath::Point3::new(0.0, 4.0, 0.0);
        let fwd = (target - eye).normalize();

        let mut camera_controller = CameraController::new(18.0, 0.003);
        camera_controller.yaw = fwd.z.atan2(fwd.x);
        camera_controller.pitch = fwd.y.asin();

        let camera = Camera::new(
            ctx,
            eye,
            target,
            Vector3::unit_y(),
            aspect,
            Projection::Perspective { fovy: 55.0 },
            0.1,
            500.0,
        );

        // Build static geometry: floor + visible wall panels
        let mut static_cubes = make_floor();
        static_cubes.extend(make_wall_panels());

        let max_dynamic = CUBE_COUNT * 8;
        let mut scene = Scene::new(ctx, camera, &static_cubes, max_dynamic);

        scene
            .set_skybox_from_bytes(
                ctx.gfx.device(),
                &ctx.gfx.queue,
                include_bytes!("sky.hdr"),
                1080,
                cgmath::SquareMatrix::identity(),
            )
            .expect("skybox");

        let mut cubes = Vec::new();
        for i in 0..CUBE_COUNT {
            let (r, g, b) = COLORS[i];
            let model = make_cube_model(r, g, b);
            let id = scene.add_dynamic(model);
            let body = RigidBody::new()
                .with_restitution(0.85) // bouncy — preserves most impact energy
                .with_friction(1.0) // less grip so collisions stay lively
                .with_half_side(CUBE_HALF)
                .with_mass(3.0) // solid feel; inertia = 2.0 kg·m²
                .with_bounds(-WALL, WALL, -WALL, WALL);

            cubes.push(Cube { id, body });
        }

        let mut demo = PhysicsDemo {
            scene,
            camera_controller,
            cursor_grabbed: false,
            cubes,
            rng_state: seed_from_time(),
        };
        demo.reset_scene();
        demo
    }

    fn update(&mut self, ctx: &mut EngineContext) {
        let dt = ctx.dt().min(0.05); // cap dt to avoid explosion on lag spikes

        // Step each cube
        for cube in &mut self.cubes {
            if let Some(model) = self.scene.get_dynamic_mut(cube.id) {
                cube.body
                    .update(&mut model.position, &mut model.rotation, dt, FLOOR_TOP);
            }
        }

        // Cube-to-cube collision
        self.resolve_collisions();

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
                        PhysicalKey::Code(KeyCode::KeyR) => {
                            self.reset_scene();
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

impl PhysicsDemo {
    fn reset_scene(&mut self) {
        // Spawn inside the play area, leaving margin so the cube never starts intersecting a wall.
        let margin = CUBE_HALF + 0.6;
        let min_xz = -WALL + margin;
        let max_xz = WALL - margin;

        // Keep initial cube centres separated so we don't start with deep penetrations.
        let min_sep = (CUBE_HALF * 2.0) * 1.35;
        let min_sep_sq = min_sep * min_sep;

        let mut spawns: Vec<Vector3<f32>> = Vec::with_capacity(self.cubes.len());
        for _ in 0..self.cubes.len() {
            let mut chosen = Vector3::new(0.0, 0.0, 0.0);
            let mut ok = false;
            for _attempt in 0..200 {
                let x = self.rand_f32(min_xz, max_xz);
                let z = self.rand_f32(min_xz, max_xz);
                let cand = Vector3::new(x, 0.0, z);

                if spawns
                    .iter()
                    .all(|p| (cand - *p).magnitude2() >= min_sep_sq)
                {
                    chosen = cand;
                    ok = true;
                    break;
                }
            }
            if !ok {
                // Fallback: accept the last candidate even if crowded.
                chosen = Vector3::new(
                    self.rand_f32(min_xz, max_xz),
                    0.0,
                    self.rand_f32(min_xz, max_xz),
                );
            }
            spawns.push(chosen);
        }

        let mut drops: Vec<f32> = Vec::with_capacity(self.cubes.len());
        let mut vels: Vec<(f32, f32)> = Vec::with_capacity(self.cubes.len());
        for _ in 0..self.cubes.len() {
            drops.push(self.rand_f32(6.0, 16.0));

            let mut vx = self.rand_f32(-10.0, 10.0);
            let mut vz = self.rand_f32(-10.0, 10.0);
            if vx.abs() + vz.abs() < 2.0 {
                vx = 6.0;
                vz = -4.0;
            }
            vels.push((vx, vz));
        }

        for (i, cube) in self.cubes.iter_mut().enumerate() {
            let drop = drops[i];
            let (vx, vz) = vels[i];
            let spawn = spawns[i];

            cube.body.linear_velocity = Vector3::new(vx, 0.0, vz);
            cube.body.angular_velocity = Vector3::new(0.0, 0.0, 0.0);
            cube.body.on_ground = false;

            if let Some(model) = self.scene.get_dynamic_mut(cube.id) {
                model.position = Vector3::new(spawn.x, FLOOR_TOP + CUBE_HALF + drop, spawn.z);
                model.rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
            }
        }
    }

    fn rand_u32(&mut self) -> u32 {
        // xorshift64* (small, fast, good enough for demo randomness)
        let mut x = self.rng_state;
        if x == 0 {
            x = 0x9e3779b97f4a7c15;
        }
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng_state = x;
        ((x.wrapping_mul(0x2545F4914F6CDD1D)) >> 32) as u32
    }

    fn rand_f32(&mut self, min: f32, max: f32) -> f32 {
        let t = (self.rand_u32() as f32) / (u32::MAX as f32);
        min + (max - min) * t
    }

    fn resolve_collisions(&mut self) {
        let n = self.cubes.len();

        // Snapshot current positions into a working buffer
        let mut positions: Vec<Vector3<f32>> = Vec::with_capacity(n);
        for cube in &self.cubes {
            let p = self
                .scene
                .get_dynamic(cube.id)
                .map(|m| m.position)
                .unwrap_or(Vector3::new(0.0, 0.0, 0.0));
            positions.push(p);
        }

        // Broad-phase + narrow-phase: O(n²), fine for n=8
        for i in 0..n {
            for j in (i + 1)..n {
                let diff = positions[i] - positions[j];
                let min_dist = (self.cubes[i].body.half_side + self.cubes[j].body.half_side) * 1.2;
                if diff.magnitude2() >= min_dist * min_dist {
                    continue;
                }

                // Split both slices at j to get four non-overlapping mut refs
                let (left, right) = self.cubes.split_at_mut(j);
                let (left_pos, right_pos) = positions.split_at_mut(j);
                physics::apply_cube_collision(
                    &mut left[i].body,
                    &mut left_pos[i],
                    &mut right[0].body,
                    &mut right_pos[0],
                );
            }
        }

        // Write corrected positions back to the scene
        for (cube, &pos) in self.cubes.iter().zip(positions.iter()) {
            if let Some(m) = self.scene.get_dynamic_mut(cube.id) {
                m.position = pos;
            }
        }
    }
}

fn seed_from_time() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x1234_5678_9abc_def0);
    nanos ^ (nanos.rotate_left(17)) ^ 0x9e37_79b9_7f4a_7c15
}

fn main() {
    env_logger::init();
    etib::run::<PhysicsDemo>(EngineConfig::default(), Some(())).expect("engine error");
}
