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

/// 8-bit sRGB channel widened to the 10-bit range the packed color uses.
const fn rgb(r: u32, g: u32, b: u32) -> UVec3 {
    UVec3::new(r * 1023 / 255, g * 1023 / 255, b * 1023 / 255)
}

/// Curated cube colors, sampled uniformly instead of random per-channel values.
const PALETTE: [UVec3; 8] = [
    rgb(0xff, 0x00, 0x00), // red
    rgb(0xff, 0x87, 0x00), // yellow
    rgb(0xff, 0xd3, 0x00), // green
    rgb(0xde, 0xff, 0x0a), // blue
    rgb(0xa1, 0xff, 0x0a), // purple
    rgb(0x0a, 0xff, 0x99), // orange
    rgb(0x0a, 0xef, 0xff), // teal
    rgb(0x14, 0x7d, 0xf5), // cream
];

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
            PALETTE[rng.random_range(0..PALETTE.len())],
        )
    }
}
