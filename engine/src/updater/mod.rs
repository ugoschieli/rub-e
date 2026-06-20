use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};
use rand::RngExt;
use std::fmt::Debug;

use crate::constants::FRAMES_IN_FLIGHT;
use crate::cube::Cube;
use crate::gfx::Gfx;
use crate::utils;

pub mod line_updater;
pub mod orbit_updater;

/// One box's full per-box data. Mirrors `BoxTransform` in the shaders
/// (rotation@0, center@16, half_extent@28, color@32, 48-byte stride).
///
/// `rotation` and `center` are *hot*: an updater's compute pass overwrites them
/// every frame. `half_extent` and `color` are *cold*: written once when the
/// buffer is created and never touched again (the updaters write only the hot
/// fields by name, so these survive in place).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct BoxTransform {
    rotation: Vec4,
    center: Vec3,
    half_extent: f32, // uniform scale on every axis, so boxes stay cubic
    color: u32,
    _pad: [u32; 3],
}

/// Per-frame uniform handed to every updater pass: the current time plus the
/// `[base, base + count)` slice of the global transform array this pass owns.
/// Mirrors `Params` in the updater shaders (16-byte uniform payload).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Params {
    time: f32,
    base: u32,
    count: u32,
    _pad: u32,
}

/// The shared per-box transform buffers, one per frame in flight. Several
/// updaters write disjoint index ranges into these, and renderers read the whole
/// thing. Owning them here (rather than in any single updater) is what lets
/// multiple updaters cooperate on the same contiguous array.
#[derive(Debug)]
pub struct TransformBuffers {
    buffers: Vec<wgpu::Buffer>,
}

impl TransformBuffers {
    pub fn new(gfx: &Gfx, cubes: &[Cube]) -> Self {
        // Seed the cold fields (size + color) once. The hot fields start at zero;
        // an updater overwrites them before the first read each frame.
        let mut rng = rand::rng();
        let init = cubes
            .iter()
            .map(|c| BoxTransform {
                rotation: Vec4::ZERO,
                center: Vec3::ZERO,
                half_extent: rng.random_range(0.2..0.5),
                color: c.color,
                _pad: [0; 3],
            })
            .collect::<Vec<BoxTransform>>();

        let buffers = (0..FRAMES_IN_FLIGHT)
            .map(|i| {
                utils::create_buffer_init(
                    &gfx.device,
                    format!("ETIB Box Transform Buffer {i}").as_str(),
                    wgpu::BufferUsages::STORAGE,
                    &init,
                )
            })
            .collect();
        Self { buffers }
    }

    /// The per-frame buffers, indexed by `gfx.frame_index`. Updaters write a
    /// sub-range; renderers bind these read-only.
    pub fn buffers(&self) -> &[wgpu::Buffer] {
        &self.buffers
    }
}

/// Produces the per-box transforms that renderers consume, decoupling *where*
/// boxes are (motion) from *how* they are drawn (rendering).
///
/// An updater writes its assigned range of the shared [`TransformBuffers`] each
/// frame. It must run before any renderer that reads them so the writes are
/// visible (a compute-pass boundary acts as the memory barrier).
pub trait Updater: Debug {
    /// Record this frame's transform update into `gfx.encoder`.
    fn update(&mut self, gfx: &mut Gfx, encoder: &mut wgpu::CommandEncoder);
}
