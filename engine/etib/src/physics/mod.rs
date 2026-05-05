//! Continuous rigid-body physics: gravity, angular momentum, impulse-based
//! collision, and corner-accurate floor/wall resolution.
//!
//! ## Quaternion rotation system
//!
//! All orientation state is stored as a **unit quaternion** (`Quaternion<f32>`).
//! Angular velocity `ω` is a world-space vector whose direction is the spin axis and
//! whose magnitude is the spin rate in rad/s.  Each frame, [`integrate_rotation`]
//! builds a delta quaternion `dq = from_axis_angle(ω̂, |ω|·dt)` and pre-multiplies
//! it onto the current orientation: `q ← (dq · q).normalize()`.  This avoids
//! gimbal lock, is numerically stable for any spin axis, and composes cleanly with
//! the impulse torques added by [`apply_cube_collision`].

use cgmath::{InnerSpace, Quaternion, Rotation, Rotation3, Vector3, Zero};

/// Gravitational acceleration in world units / s².
/// Set higher than real gravity (9.81) to give cubes authority on landing.
pub const GRAVITY: f32 = 32.0;

/// Air resistance fraction removed per second from horizontal and angular motion.
/// Keeps cubes from sliding infinitely on a perfectly frictionless surface.
const AIR_DAMPING: f32 = 0.2;

/// Below this vertical speed a floor bounce is suppressed and the cube sticks.
/// Higher value = cube stops bouncing sooner, feels denser.
const BOUNCE_STOP: f32 = 0.25;

/// Horizontal speed below which the settling phase kicks in (m/s).
const SETTLE_LIN_THRESH: f32 = 0.35;
/// Angular speed below which the settling phase kicks in (rad/s).
const SETTLE_ANG_THRESH: f32 = 0.75;
/// Fraction of the remaining angle to cover per second during settling.
const SETTLE_RATE: f32 = 5.0;
/// Extra velocity damping coefficient applied while settling.
const SETTLE_DAMP: f32 = 3.0;
/// Angle (radians) below which the cube is snapped flat and frozen.
const SETTLE_SNAP_ANGLE: f32 = 0.015;
/// Speed (m/s, rad/s) below which velocities are zeroed on snap.
const SETTLE_SNAP_VEL: f32 = 0.04;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// The 8 corner offsets of a cube with given half-side.
fn corners(h: f32) -> [Vector3<f32>; 8] {
    [
        Vector3::new(-h, -h, -h),
        Vector3::new( h, -h, -h),
        Vector3::new(-h,  h, -h),
        Vector3::new( h,  h, -h),
        Vector3::new(-h, -h,  h),
        Vector3::new( h, -h,  h),
        Vector3::new(-h,  h,  h),
        Vector3::new( h,  h,  h),
    ]
}

/// Return the nearest of the 24 valid face-aligned cube orientations to `q`.
///
/// Used during settling to find the target orientation the cube should tip onto.
fn nearest_face_rotation(q: Quaternion<f32>) -> Quaternion<f32> {
    use cgmath::Rad;
    use std::f32::consts::{FRAC_PI_2, PI};

    let faces: [Quaternion<f32>; 6] = [
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
        Quaternion::from_axis_angle(Vector3::unit_x(), Rad(FRAC_PI_2)),
        Quaternion::from_axis_angle(Vector3::unit_x(), Rad(-FRAC_PI_2)),
        Quaternion::from_axis_angle(Vector3::unit_x(), Rad(PI)),
        Quaternion::from_axis_angle(Vector3::unit_z(), Rad(FRAC_PI_2)),
        Quaternion::from_axis_angle(Vector3::unit_z(), Rad(-FRAC_PI_2)),
    ];
    let spins: [Quaternion<f32>; 4] = [
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
        Quaternion::from_axis_angle(Vector3::unit_y(), Rad(FRAC_PI_2)),
        Quaternion::from_axis_angle(Vector3::unit_y(), Rad(PI)),
        Quaternion::from_axis_angle(Vector3::unit_y(), Rad(-FRAC_PI_2)),
    ];

    let mut best = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    let mut best_dot = f32::NEG_INFINITY;
    for &face in &faces {
        for &spin in &spins {
            let c = (face * spin).normalize();
            let dot = (c.s * q.s + c.v.dot(q.v)).abs();
            if dot > best_dot {
                best_dot = dot;
                best = c;
            }
        }
    }
    best
}

/// Angle in radians between two unit quaternions (treating q and −q as equal).
fn quat_angle(a: Quaternion<f32>, b: Quaternion<f32>) -> f32 {
    let dot = (a.s * b.s + a.v.dot(b.v)).abs().min(1.0);
    2.0 * dot.acos()
}

/// Integrate angular velocity `omega` (rad/s, world-space) into a quaternion for `dt` seconds.
///
/// Builds a delta quaternion `dq = from_axis_angle(ω̂, |ω|·dt)` then
/// updates `rotation ← (dq · rotation).normalize()`.
/// Pre-multiplying keeps the spin in world space, matching how angular
/// impulses from collisions are expressed.
fn integrate_rotation(rotation: &mut Quaternion<f32>, omega: Vector3<f32>, dt: f32) {
    let speed = omega.magnitude();
    if speed < 1e-5 {
        return;
    }
    let axis = omega / speed;
    let dq = Quaternion::from_axis_angle(axis, cgmath::Rad(speed * dt));
    // Pre-multiply: dq is in world space, rotation maps local→world.
    *rotation = (dq * *rotation).normalize();
}

// ---------------------------------------------------------------------------
// RigidBody
// ---------------------------------------------------------------------------

/// Continuous rigid-body state for a cube.
pub struct RigidBody {
    /// Linear velocity in world units / s.
    pub linear_velocity: Vector3<f32>,
    /// Angular velocity (axis × rad/s).
    pub angular_velocity: Vector3<f32>,
    /// Bounce coefficient [0, 1].
    pub restitution: f32,
    /// Coulomb friction coefficient used for surface contacts (floor + other cubes).
    pub floor_friction: f32,
    /// Half the cube side (centre to face).
    pub half_side: f32,
    /// Mass in kg.
    pub mass: f32,
    /// True while the cube is in contact with the floor.
    pub on_ground: bool,
    /// Wall bounds [x_min, x_max, z_min, z_max].
    pub bounds: [f32; 4],
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            linear_velocity: Vector3::zero(),
            angular_velocity: Vector3::zero(),
            restitution: 0.20,   // dense hardwood — absorbs most impact energy
            floor_friction: 1.0, // Coulomb μ; 0=ice, ~1=grippy rubber
            half_side: 1.0,
            mass: 3.0,           // heavier cube; inertia = m·a²/6 = 3·4/6 = 2.0 kg·m²
            on_ground: false,
            bounds: [f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY],
        }
    }
}

impl RigidBody {
    /// Create a body with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set bounce coefficient (builder style).
    pub fn with_restitution(mut self, r: f32) -> Self {
        self.restitution = r.clamp(0.0, 1.0);
        self
    }

    /// Set Coulomb friction coefficient μ for contacts (builder style).
    pub fn with_friction(mut self, f: f32) -> Self {
        self.floor_friction = f.max(0.0);
        self
    }

    /// Set half-side length (builder style).
    pub fn with_half_side(mut self, h: f32) -> Self {
        self.half_side = h.max(0.001);
        self
    }

    /// Set mass in kg (builder style).
    pub fn with_mass(mut self, m: f32) -> Self {
        self.mass = m.max(0.001);
        self
    }

    /// Set invisible wall bounds (builder style).
    pub fn with_bounds(mut self, x_min: f32, x_max: f32, z_min: f32, z_max: f32) -> Self {
        self.bounds = [x_min, x_max, z_min, z_max];
        self
    }

    /// Moment of inertia for a uniform solid cube rotating about an axis
    /// through its centre: I = m·a² / 6  where a = side length.
    pub fn inertia(&self) -> f32 {
        let a = self.half_side * 2.0;
        self.mass * a * a / 6.0
    }

    /// Advance one frame.
    pub fn update(
        &mut self,
        position: &mut Vector3<f32>,
        rotation: &mut Quaternion<f32>,
        dt: f32,
        floor_y: f32,
    ) {
        // Gravity (only on Y)
        self.linear_velocity.y -= GRAVITY * dt;

        // Light air damping on horizontal and angular motion
        let ad = (1.0 - AIR_DAMPING * dt).max(0.0);
        self.linear_velocity.x *= ad;
        self.linear_velocity.z *= ad;
        self.angular_velocity *= ad;

        // Integrate position and orientation
        *position += self.linear_velocity * dt;
        integrate_rotation(rotation, self.angular_velocity, dt);

        // Floor collision
        self.resolve_floor(position, rotation, floor_y, dt);

        // Wall collision
        self.resolve_walls(position, rotation);

        // Face settling — smoothly tips the cube onto its nearest face once slow
        if self.on_ground {
            self.settle_to_face(position, rotation, floor_y, dt);
        }
    }

    fn settle_to_face(
        &mut self,
        position: &mut Vector3<f32>,
        rotation: &mut Quaternion<f32>,
        floor_y: f32,
        dt: f32,
    ) {
        let horiz_sq = self.linear_velocity.x * self.linear_velocity.x
            + self.linear_velocity.z * self.linear_velocity.z;
        let ang_sq = self.angular_velocity.magnitude2();

        // Only settle when both linear and angular motion are slow enough
        if horiz_sq > SETTLE_LIN_THRESH * SETTLE_LIN_THRESH
            || ang_sq > SETTLE_ANG_THRESH * SETTLE_ANG_THRESH
        {
            return;
        }

        // Find the nearest valid face-aligned orientation
        let target = nearest_face_rotation(*rotation);

        // Slerp toward it — the blend fraction is capped so motion stays smooth
        let t = (SETTLE_RATE * dt).min(0.5);
        *rotation = rotation.slerp(target, t).normalize();

        // Extra damping while settling so the cube bleeds energy naturally
        let damp = (1.0 - SETTLE_DAMP * dt).max(0.0);
        self.linear_velocity *= damp;
        self.angular_velocity *= damp;

        // As the cube rotates, keep its lowest corner exactly on the floor
        // so it appears to roll onto the face rather than sink or float.
        let min_rot_y = corners(self.half_side)
            .iter()
            .map(|&c| rotation.rotate_vector(c).y)
            .fold(f32::INFINITY, f32::min);
        position.y = floor_y - min_rot_y;

        // Once close enough, snap perfectly flat and zero all motion
        if quat_angle(*rotation, target) < SETTLE_SNAP_ANGLE
            && horiz_sq < SETTLE_SNAP_VEL * SETTLE_SNAP_VEL
            && ang_sq < SETTLE_SNAP_VEL * SETTLE_SNAP_VEL
        {
            *rotation = target;
            self.linear_velocity = Vector3::zero();
            self.angular_velocity = Vector3::zero();

            // Recompute floor contact with the exact final orientation
            let min_exact_y = corners(self.half_side)
                .iter()
                .map(|&c| rotation.rotate_vector(c).y)
                .fold(f32::INFINITY, f32::min);
            position.y = floor_y - min_exact_y;
        }
    }

    fn resolve_floor(
        &mut self,
        position: &mut Vector3<f32>,
        rotation: &Quaternion<f32>,
        floor_y: f32,
        dt: f32,
    ) {
        let h = self.half_side;
        let world_corners: [Vector3<f32>; 8] = corners(h).map(|c| *position + rotation.rotate_vector(c));
        let min_y = world_corners
            .iter()
            .map(|w| w.y)
            .fold(f32::INFINITY, f32::min);

        if min_y < floor_y {
            // Push up out of floor
            position.y += floor_y - min_y;
            self.on_ground = true;

            // Approximate contact patch: average all corners that are (nearly) at the lowest Y.
            // This gives a surface point (not the centre) so impulses generate torque.
            let eps = 1e-3;
            let mut contact = Vector3::zero();
            let mut count = 0.0;
            for w in &world_corners {
                if (w.y - min_y).abs() <= eps {
                    contact += *w;
                    count += 1.0;
                }
            }
            if count > 0.0 {
                contact /= count;
            } else {
                // Fallback (shouldn't happen): use the first corner.
                contact = world_corners[0];
            }
            contact.y = floor_y;

            let n = Vector3::unit_y();
            let r = contact - *position;
            let inv_m = 1.0 / self.mass;
            let i = self.inertia();

            // --- Normal impulse (bounce) applied at the contact point ---
            // Includes rotational contribution so angled hits tumble correctly.
            let v_contact = self.linear_velocity + self.angular_velocity.cross(r);
            let v_n = v_contact.dot(n);

            // Normal impulse magnitude used for Coulomb friction clamping.
            let jn_mag: f32;

            if v_n < 0.0 {
                let rn = r.cross(n);
                let denom_n = inv_m + rn.dot(rn) / i;
                let jn = -(1.0 + self.restitution) * v_n / denom_n;
                let impulse_n = n * jn;

                self.linear_velocity += impulse_n * inv_m;
                self.angular_velocity += r.cross(impulse_n) / i;
                jn_mag = jn.abs();

                // Small-bounce suppression: if the post-collision normal speed is tiny, stick.
                let v_post = self.linear_velocity + self.angular_velocity.cross(r);
                let v_post_n = v_post.dot(n);
                if v_post_n.abs() < BOUNCE_STOP {
                    // Cancel remaining normal motion at the contact.
                    let j_cancel = -v_post_n / denom_n;
                    let impulse_cancel = n * j_cancel;
                    self.linear_velocity += impulse_cancel * inv_m;
                    self.angular_velocity += r.cross(impulse_cancel) / i;
                }
            } else {
                // Resting contact: approximate the per-frame normal impulse from gravity.
                jn_mag = (self.mass * GRAVITY * dt).max(0.0);
            }

            // --- Tangential friction impulse (drives spin) ---
            // Opposes the relative surface velocity at the contact point.
            let v_contact = self.linear_velocity + self.angular_velocity.cross(r);
            let v_t = v_contact - n * v_contact.dot(n);
            let vt_mag = v_t.magnitude();
            if vt_mag > 1e-4 {
                let t = v_t / vt_mag;
                let rt = r.cross(t);
                let denom_t = inv_m + rt.dot(rt) / i;

                // Impulse to bring tangential contact velocity toward zero (static friction target)
                let mut jt = -v_t.dot(t) / denom_t;
                let jt_max = self.floor_friction * jn_mag;
                jt = jt.clamp(-jt_max, jt_max);

                let impulse_t = t * jt;
                self.linear_velocity += impulse_t * inv_m;
                self.angular_velocity += r.cross(impulse_t) / i;
            }
        } else {
            self.on_ground = false;
        }
    }

    fn resolve_walls(&mut self, position: &mut Vector3<f32>, rotation: &Quaternion<f32>) {
        let [x_min, x_max, z_min, z_max] = self.bounds;
        let h = self.half_side;

        // Compute rotated extents once
        let (mut rmax_x, mut rmin_x, mut rmax_z, mut rmin_z) =
            (f32::NEG_INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY);
        for &c in &corners(h) {
            let w = rotation.rotate_vector(c);
            rmax_x = rmax_x.max(w.x);
            rmin_x = rmin_x.min(w.x);
            rmax_z = rmax_z.max(w.z);
            rmin_z = rmin_z.min(w.z);
        }

        if position.x + rmax_x > x_max {
            position.x = x_max - rmax_x;
            if self.linear_velocity.x > 0.0 {
                self.linear_velocity.x = -self.linear_velocity.x * self.restitution;
                // Wall imparts a small spin in proportion to impact speed
                self.angular_velocity.z -= self.linear_velocity.x.abs() * 0.2;
            }
        }
        if position.x + rmin_x < x_min {
            position.x = x_min - rmin_x;
            if self.linear_velocity.x < 0.0 {
                self.linear_velocity.x = -self.linear_velocity.x * self.restitution;
                self.angular_velocity.z += self.linear_velocity.x.abs() * 0.2;
            }
        }
        if position.z + rmax_z > z_max {
            position.z = z_max - rmax_z;
            if self.linear_velocity.z > 0.0 {
                self.linear_velocity.z = -self.linear_velocity.z * self.restitution;
                self.angular_velocity.x += self.linear_velocity.z.abs() * 0.2;
            }
        }
        if position.z + rmin_z < z_min {
            position.z = z_min - rmin_z;
            if self.linear_velocity.z < 0.0 {
                self.linear_velocity.z = -self.linear_velocity.z * self.restitution;
                self.angular_velocity.x -= self.linear_velocity.z.abs() * 0.2;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cube-to-cube impulse collision
// ---------------------------------------------------------------------------

/// Resolve a collision between two cubes using impulse-based rigid-body physics.
///
/// Detects contact using a sphere approximation (circumradius = `half_side × √3`),
/// then computes a physically correct impulse that affects both linear and angular
/// velocity proportionally to the impact angle and speed.
///
/// Call this every frame for every pair of cubes that may be close.
pub fn apply_cube_collision(
    a: &mut RigidBody,
    pos_a: &mut Vector3<f32>,
    b: &mut RigidBody,
    pos_b: &mut Vector3<f32>,
) {
    let diff = *pos_a - *pos_b;
    let dist_sq = diff.magnitude2();
    if dist_sq < 1e-8 {
        return;
    }

    // Contact threshold: use the sum of face-diagonal radii for a good
    // balance between catching corner hits and avoiding phantom collisions.
    let r_a = a.half_side * 1.2;
    let r_b = b.half_side * 1.2;
    let min_dist = r_a + r_b;

    let dist = dist_sq.sqrt();
    if dist >= min_dist {
        return;
    }

    // Collision normal: from B toward A
    let n = diff / dist;
    let penetration = min_dist - dist;

    // Push the two cubes apart proportional to inverse mass
    let inv_a = 1.0 / a.mass;
    let inv_b = 1.0 / b.mass;
    let share = 1.0 / (inv_a + inv_b);
    *pos_a += n * (penetration * inv_a * share);
    *pos_b -= n * (penetration * inv_b * share);

    // Contact point (approximate midpoint on the collision surface)
    let contact = (*pos_a + *pos_b) * 0.5;
    let r_vec_a = contact - *pos_a;
    let r_vec_b = contact - *pos_b;

    // Relative velocity at the contact point (includes rotational contribution)
    let v_a = a.linear_velocity + a.angular_velocity.cross(r_vec_a);
    let v_b = b.linear_velocity + b.angular_velocity.cross(r_vec_b);
    let v_rel = v_a - v_b;
    let v_rel_n = v_rel.dot(n);

    // Already separating — nothing to do
    if v_rel_n >= 0.0 {
        return;
    }

    let e = (a.restitution + b.restitution) * 0.5;
    let i_a = a.inertia();
    let i_b = b.inertia();

    // Angular contribution to the effective mass at the contact point
    let ra_x_n = r_vec_a.cross(n);
    let rb_x_n = r_vec_b.cross(n);
    let ang = ra_x_n.dot(ra_x_n) / i_a + rb_x_n.dot(rb_x_n) / i_b;
    let denom = inv_a + inv_b + ang;

    // Impulse magnitude
    let j = -(1.0 + e) * v_rel_n / denom;
    let impulse = n * j;

    // Apply linear impulse
    a.linear_velocity += impulse * inv_a;
    b.linear_velocity -= impulse * inv_b;

    // Apply angular impulse — this is what makes cubes spin on oblique hits
    a.angular_velocity += r_vec_a.cross(impulse) / i_a;
    b.angular_velocity -= r_vec_b.cross(impulse) / i_b;

    // --- Tangential friction impulse (drives spin on sliding/angled impacts) ---
    // Recompute contact velocity after the normal impulse for stability.
    let v_a2 = a.linear_velocity + a.angular_velocity.cross(r_vec_a);
    let v_b2 = b.linear_velocity + b.angular_velocity.cross(r_vec_b);
    let v_rel2 = v_a2 - v_b2;
    let v_rel2_n = v_rel2.dot(n);
    let v_t = v_rel2 - n * v_rel2_n;
    let vt_mag = v_t.magnitude();
    if vt_mag > 1e-4 {
        let t = v_t / vt_mag;
        let ra_x_t = r_vec_a.cross(t);
        let rb_x_t = r_vec_b.cross(t);
        let ang_t = ra_x_t.dot(ra_x_t) / i_a + rb_x_t.dot(rb_x_t) / i_b;
        let denom_t = inv_a + inv_b + ang_t;

        let mut jt = -v_rel2.dot(t) / denom_t;
        let mu = (a.floor_friction + b.floor_friction) * 0.5;
        let jt_max = mu * j.abs();
        jt = jt.clamp(-jt_max, jt_max);

        let impulse_t = t * jt;
        a.linear_velocity += impulse_t * inv_a;
        b.linear_velocity -= impulse_t * inv_b;
        a.angular_velocity += r_vec_a.cross(impulse_t) / i_a;
        b.angular_velocity -= r_vec_b.cross(impulse_t) / i_b;
    }
}
