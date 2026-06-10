use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Quat, Vec3};
use rand::RngExt;
use std::f32::consts::TAU;
use std::time::Instant;
use wgpu::include_wgsl;

use crate::constants::FRAMES_IN_FLIGHT;
use crate::cube::Cube;
use crate::gfx::Gfx;
use crate::updater::{Params, Updater};
use crate::utils::{self, bindgroup::FrameBuffered};

/// Cold, static orbit of one box. Uploaded once at init; the GPU update pass
/// derives the per-frame position/orientation from this plus the time uniform.
/// Mirrors `Orbit` in `shaders/orbit.wgsl` (48-byte stride).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Orbit {
    center: Vec3, // orbit center in raw world space
    radius: f32,
    u: Vec3, // orthonormal in-plane basis vectors
    speed: f32, // angular speed, rad/s (signed for direction)
    v: Vec3,
    phase: f32, // initial angle
}

/// Flies its assigned boxes along their own fixed circular orbits, on the GPU.
///
/// Owns the orbit parameters (uploaded once) and writes the `[base, base + count)`
/// range of the shared transform buffers each frame from one time uniform, so
/// per-frame CPU work and upload are constant-size regardless of box count.
#[derive(Debug)]
pub struct OrbitUpdater {
    pipeline: wgpu::ComputePipeline,
    bind_group: FrameBuffered,
    param_buffers: Vec<wgpu::Buffer>,
    base: u32,
    count: u32,
    start: Instant,
}

impl OrbitUpdater {
    /// Assign `cubes` (a sub-slice of all boxes) to circular orbits, writing into
    /// the global transform array starting at `base`. `transform_buffers` are the
    /// shared per-frame buffers from [`TransformBuffers`](crate::updater::TransformBuffers).
    pub fn init(gfx: &Gfx, cubes: &[Cube], transform_buffers: &[wgpu::Buffer], base: u32) -> Self {
        // A random circular orbit per box: the cube's spawn position is the
        // orbit center, and (u, v) is a random orthonormal in-plane basis taken
        // from a random orientation.
        let mut rng = rand::rng();
        let orbits = cubes
            .iter()
            .map(|c| {
                let plane = Quat::from_euler(
                    EulerRot::XYZ,
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                );
                Orbit {
                    center: c.position.as_vec3() + 0.5,
                    radius: rng.random_range(2.0..30.0),
                    u: plane * Vec3::X,
                    speed: rng.random_range(-0.5..0.5),
                    v: plane * Vec3::Y,
                    phase: rng.random_range(0.0..TAU),
                }
            })
            .collect::<Vec<Orbit>>();

        let count = cubes.len() as u32;

        let orbit_buffer = utils::create_buffer_init(
            &gfx.device,
            "ETIB Orbit Buffer",
            wgpu::BufferUsages::STORAGE,
            &orbits,
        );

        let mut param_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        for i in 0..FRAMES_IN_FLIGHT {
            param_buffers.push(utils::create_buffer_init(
                &gfx.device,
                format!("ETIB Orbit Params Buffer {i}").as_str(),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                &[Params {
                    time: 0.0,
                    base,
                    count,
                    _pad: 0,
                }],
            ));
        }

        // params (uniform), orbits (read), shared transforms (write).
        let layout = utils::bindgroup::BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::COMPUTE)
            .uniform(0) // Params (time + range)
            .storage(1, true) // Orbits (cold)
            .storage(2, false) // Shared box transforms (hot, written)
            .build();

        let bind_group = FrameBuffered::new(&gfx.device, layout, |i| {
            vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: param_buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: orbit_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transform_buffers[i].as_entire_binding(),
                },
            ]
        });

        let shader = gfx
            .device
            .create_shader_module(include_wgsl!("../../shaders/orbit.wgsl"));

        let pipeline = utils::create_compute_pipeline_from_module(
            &gfx.device,
            "ETIB Orbit Update",
            &bind_group.layout,
            &shader,
        );

        Self {
            pipeline,
            bind_group,
            param_buffers,
            base,
            count,
            start: Instant::now(),
        }
    }
}

impl Updater for OrbitUpdater {
    fn update(&mut self, gfx: &mut Gfx) {
        // The only per-frame upload: elapsed seconds driving every orbit.
        let time = self.start.elapsed().as_secs_f32();
        gfx.queue.write_buffer(
            &self.param_buffers[gfx.frame_index],
            0,
            bytemuck::bytes_of(&Params {
                time,
                base: self.base,
                count: self.count,
                _pad: 0,
            }),
        );

        let mut encoder = gfx.encoder.as_mut().unwrap();
        let mut pass = utils::create_compute_pass(&mut encoder);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bind_group.current(gfx.frame_index), &[]);
        pass.dispatch_workgroups((self.count as usize).div_ceil(64) as u32, 1, 1);
    }
}
