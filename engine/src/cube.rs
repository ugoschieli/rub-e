use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4, Vec4Swizzles};
use rand::RngExt;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub struct Cube {
    pub position: Vec3,
    pub color: Vec3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CubeGpu {
    pub position: Vec4,
    pub color: Vec4,
}

impl Cube {
    pub const fn new(position: Vec3, color: Vec3) -> Self {
        Self { position, color }
    }

    pub fn to_gpu(self) -> CubeGpu {
        CubeGpu {
            position: Vec4::ZERO.with_xyz(self.position),
            color: Vec4::ONE.with_xyz(self.color),
        }
    }

    pub fn random_cube(range: Range<f32>) -> Self {
        let mut rng = rand::rng();
        Self {
            position: Vec3::new(
                rng.random_range(range.clone()),
                rng.random_range(range.clone()),
                rng.random_range(range),
            ),

            color: Vec3::new(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
            ),
        }
    }
}
