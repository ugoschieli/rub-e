use std::ops::Range;

pub const CUBE_NUMBER: usize = 1_000_000;
pub const CUBE_RANGE: Range<i32> = -128..128;

pub const SENSITIVITY: f32 = 0.05;
pub const SPEED: f32 = 3.0;

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub const FRAMES_IN_FLIGHT: usize = 2;

pub const CHUNK_SIZE_1: usize = 62;
pub const CHUNK_SIZE_2: usize = CHUNK_SIZE_1 * CHUNK_SIZE_1;
pub const CHUNK_SIZE_3: usize = CHUNK_SIZE_1 * CHUNK_SIZE_1 * CHUNK_SIZE_1;
pub const CHUNK_SIZE_P: usize = CHUNK_SIZE_1 + 2;
pub const CHUNK_SIZE_P2: usize = CHUNK_SIZE_P * CHUNK_SIZE_P;
pub const CHUNK_SIZE_P3: usize = CHUNK_SIZE_P * CHUNK_SIZE_P * CHUNK_SIZE_P;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
