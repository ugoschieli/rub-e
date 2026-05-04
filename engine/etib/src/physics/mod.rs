//! Simple rigid-body physics: gravity integration and floor collision.

use cgmath::{Vector3, Zero};

/// Downward acceleration in world units per second squared.
pub const GRAVITY: f32 = 20.0;

/// Velocity threshold below which a bouncing body is considered at rest.
const REST_THRESHOLD: f32 = 0.5;

/// Physics state for a single dynamic body subject to gravity.
pub struct RigidBody {
    /// Current velocity in world units per second.
    pub velocity: Vector3<f32>,
    /// Energy retained after each floor bounce (0 = no bounce, 1 = perfect bounce).
    pub restitution: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            velocity: Vector3::zero(),
            restitution: 0.5,
        }
    }
}

impl RigidBody {
    /// Create a new body with zero velocity and restitution 0.5.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style restitution setter.
    pub fn with_restitution(mut self, r: f32) -> Self {
        self.restitution = r.clamp(0.0, 1.0);
        self
    }

    /// Apply gravity for `dt` seconds and return the position delta to add.
    ///
    /// Call this once per frame, then add the returned delta to the body's
    /// world position, then call [`resolve_floor`] to handle collisions.
    pub fn step(&mut self, dt: f32) -> Vector3<f32> {
        self.velocity.y -= GRAVITY * dt;
        self.velocity * dt
    }

    /// Resolve a collision against a flat horizontal floor.
    ///
    /// `floor_y` — world-space Y of the floor surface.
    /// `half_height` — distance from the body's origin to its bottom face.
    ///
    /// If the body's bottom has passed through the floor, it is pushed back up
    /// and its vertical velocity is reflected and damped by `restitution`.
    pub fn resolve_floor(
        &mut self,
        position: &mut Vector3<f32>,
        floor_y: f32,
        half_height: f32,
    ) {
        let bottom = position.y - half_height;
        if bottom <= floor_y {
            position.y = floor_y + half_height;
            if self.velocity.y < 0.0 {
                self.velocity.y = -self.velocity.y * self.restitution;
                if self.velocity.y < REST_THRESHOLD {
                    self.velocity.y = 0.0;
                }
            }
        }
    }
}
