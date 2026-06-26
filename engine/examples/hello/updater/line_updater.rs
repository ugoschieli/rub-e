use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Quat, Vec3};
use rand::RngExt;
use std::f32::consts::TAU;
use std::time::Instant;
use wgpu::include_wgsl;

use super::{Params, Updater};
use etib::constants::FRAMES_IN_FLIGHT;
use etib::core::bind_group::FrameBuffered;
use etib::cube::Cube;
use etib::gfx::Gfx;
use etib::utils::{self};

/// Cold, static line of one box. Uploaded once at init; the GPU update pass
/// derives the per-frame position/orientation from this plus the time uniform.
/// Mirrors `Line` in `shaders/line.wgsl` (48-byte stride).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Line {
    anchor: Vec3,   // line midpoint in raw world space
    amplitude: f32, // half-length of the travel
    dir: Vec3,      // unit travel direction
    speed: f32,     // oscillation angular speed, rad/s
    axis: Vec3,     // unit spin axis
    phase: f32,     // initial angle
}

/// Slides its assigned boxes back and forth along their own fixed straight
/// lines, on the GPU.
///
/// Owns the line parameters (uploaded once) and writes the `[base, base + count]`
/// range of the shared transform buffers each frame from one time uniform.
#[derive(Debug)]
pub struct LineUpdater {
    pipeline: wgpu::ComputePipeline,
    bind_group: FrameBuffered,
    param_buffers: Vec<wgpu::Buffer>,
    base: u32,
    count: u32,
    start: Instant,
}

impl LineUpdater {
    /// Assign `cubes` (a sub-slice of all boxes) to oscillating lines, writing
    /// into the global transform array starting at `base`. `transform_buffers`
    /// are the shared per-frame buffers from
    /// [`TransformBuffers`](crate::updater::TransformBuffers).
    pub fn init(gfx: &Gfx, cubes: &[Cube], transform_buffers: &[wgpu::Buffer], base: u32) -> Self {
        // A random line per box: the cube's spawn position is the line midpoint,
        // with a random travel direction and an independent random spin axis.
        let mut rng = rand::rng();
        let lines = cubes
            .iter()
            .map(|c| {
                let dir = Quat::from_euler(
                    EulerRot::XYZ,
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                ) * Vec3::X;
                let axis = Quat::from_euler(
                    EulerRot::XYZ,
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                    rng.random_range(0.0..TAU),
                ) * Vec3::Y;
                Line {
                    anchor: c.position.as_vec3() + 0.5,
                    amplitude: rng.random_range(2.0..30.0),
                    dir,
                    speed: rng.random_range(0.2..0.5),
                    axis,
                    phase: rng.random_range(0.0..TAU),
                }
            })
            .collect::<Vec<Line>>();

        let count = u32::try_from(cubes.len()).unwrap();

        let line_buffer = utils::create_buffer_init(
            &gfx.device,
            "ETIB Line Buffer",
            wgpu::BufferUsages::STORAGE,
            &lines,
        );

        let mut param_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        for i in 0..FRAMES_IN_FLIGHT {
            param_buffers.push(utils::create_buffer_init(
                &gfx.device,
                format!("ETIB Line Params Buffer {i}").as_str(),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                &[Params {
                    time: 0.0,
                    base,
                    count,
                    _pad: 0,
                }],
            ));
        }

        // params (uniform), lines (read), shared transforms (write).
        let layout = etib::core::bind_group::BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::COMPUTE)
            .uniform(0) // Params (time + range)
            .storage(1, true) // Lines (cold)
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
                    resource: line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transform_buffers[i].as_entire_binding(),
                },
            ]
        });

        let shader = gfx
            .device
            .create_shader_module(include_wgsl!("../shaders/line.wgsl"));

        let pipeline =
            gfx.create_compute_pipeline("ETIB Line Update", &bind_group.layout, 0, &shader);

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

impl Updater for LineUpdater {
    fn update(&mut self, gfx: &mut Gfx, encoder: &mut wgpu::CommandEncoder) {
        // The only per-frame upload: elapsed seconds driving every line.
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

        let mut pass = gfx.create_compute_pass("line_update compute pass", encoder);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, self.bind_group.current(gfx.frame_index), &[]);
        pass.dispatch_workgroups(self.count.div_ceil(64), 1, 1);
    }
}
