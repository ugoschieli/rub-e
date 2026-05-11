use bytemuck::{Pod, Zeroable};
use glam::{IVec3, UVec3};
use rand::RngExt;
use std::ops::Range;

#[derive(Debug, Copy, Clone, Default)]
pub struct Cube {
    pub position: IVec3,
    pub color: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CubeGpu {
    pub position: IVec3,
    pub color: u32,
}

impl Cube {
    pub const fn new(position: IVec3, color: UVec3) -> Self {
        let color = color.x | (color.y << 10) | (color.z << 20); // packed color (10 bit for each channel)
        Self { position, color }
    }

    pub const fn to_gpu(self) -> CubeGpu {
        CubeGpu {
            position: self.position,
            color: self.color,
        }
    }

    pub fn random_cube(range: Range<i32>) -> Self {
        let mut rng = rand::rng();
        Self::new(
            IVec3::new(
                rng.random_range(range.clone()),
                rng.random_range(range.clone()),
                rng.random_range(range),
            ),
            UVec3::new(
                rng.random_range(0..1023),
                rng.random_range(0..1023),
                rng.random_range(0..1023),
            ),
        )
    }
}
