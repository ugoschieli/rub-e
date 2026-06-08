use wgpu::include_wgsl;
use winit::dpi::PhysicalSize;

use crate::camera::Camera;
use crate::chunk::World;
use crate::constants::FRAMES_IN_FLIGHT;
use crate::cube::Cube;
use crate::gfx::Gfx;
use crate::renderer::Renderer;
use crate::utils::{self, bindgroup::FrameBuffered};

/// Number of vertices in the unit-box proxy (12 triangles). This is the constant
/// `vertex_count` of every indirect draw issued by this renderer.
const BOX_VERTEX_COUNT: u32 = 36;

/// Renders each voxel as one instanced oriented-box proxy and recovers the exact
/// surface hit + normal in the fragment shader via the ray-box intersection
/// algorithm (see `shaders/ray_box.wgsl`).
///
/// Two GPU passes run per frame:
/// 1. `shaders/box_cull.wgsl` frustum-culls every box and compacts the survivors
///    into a per-frame index list plus an indirect draw whose `instance_count`
///    is the survivor count.
/// 2. The render pass draws only the visible boxes via that indirect draw.
///
/// All per-box data (position, rotation, size, color) lives in the transform
/// buffers produced by a separate [`Updater`](crate::updater::Updater); this
/// renderer only reads them and owns no per-box buffers of its own.
#[derive(Debug)]
pub struct DynamicRenderer {
    cull_pipeline: wgpu::ComputePipeline,
    cull_bind_group: FrameBuffered,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: FrameBuffered,
    camera_pos_buffers: Vec<wgpu::Buffer>,
    // Per frame-in-flight: [vertex_count, instance_count, first_vertex, first_instance].
    // `instance_count` is reset to 0 and then filled by the cull pass each frame.
    draw_args_buffers: Vec<wgpu::Buffer>,
    cube_count: usize,
}

impl DynamicRenderer {
    /// `transform_buffers` are the per-frame hot transforms written by an
    /// [`Updater`](crate::updater::Updater); one per frame in flight, indexed by
    /// `gfx.frame_index`. The updater must run before this renderer each frame.
    pub fn init(
        gfx: &Gfx,
        camera: &Camera,
        cubes: &[Cube],
        transform_buffers: &[wgpu::Buffer],
    ) -> Self {
        let cube_count = cubes.len();

        let mut camera_pos_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        let mut draw_args_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        let mut visible_index_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        for i in 0..FRAMES_IN_FLIGHT {
            camera_pos_buffers.push(utils::create_buffer_init(
                &gfx.device,
                format!("ETIB Dynamic Camera Pos Buffer {i}").as_str(),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                &[glam::Vec4::ZERO],
            ));

            // vertex_count fixed at the box proxy size; instance_count starts at 0
            // and is rebuilt by the cull pass every frame.
            draw_args_buffers.push(utils::create_buffer_init(
                &gfx.device,
                format!("ETIB Dynamic Draw Args Buffer {i}").as_str(),
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_DST,
                &[BOX_VERTEX_COUNT, 0, 0, 0],
            ));

            // Compacted survivor indices; at most one entry per box.
            visible_index_buffers.push(utils::create_buffer(
                &gfx.device,
                format!("ETIB Dynamic Visible Index Buffer {i}").as_str(),
                cube_count as u64 * size_of::<u32>() as u64,
                wgpu::BufferUsages::STORAGE,
            ));
        }

        // Compute cull pass: view-projection (uniform), transforms (read), the
        // indirect draw args (read_write, atomic counter), and the survivor list.
        let cull_layout = utils::bindgroup::BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::COMPUTE)
            .uniform(0) // Camera view-projection matrix
            .storage(1, true) // Box transforms (from the updater)
            .storage(2, false) // Indirect draw args (atomic instance_count)
            .storage(3, false) // Compacted visible indices
            .build();

        let cull_bind_group = FrameBuffered::new(&gfx.device, cull_layout, |i| {
            vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: transform_buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: draw_args_buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: visible_index_buffers[i].as_entire_binding(),
                },
            ]
        });

        let cull_shader = gfx
            .device
            .create_shader_module(include_wgsl!("../../shaders/renderer/dynamic/box_cull.wgsl"));

        let cull_pipeline = utils::create_compute_pipeline_from_module(
            &gfx.device,
            "ETIB Dynamic Cull",
            &cull_bind_group.layout,
            &cull_shader,
        );

        // Render pass: camera + the transforms plus the survivor list, which the
        // vertex shader indexes by `instance_index` to recover the real box.
        let render_layout = utils::bindgroup::BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::VERTEX_FRAGMENT)
            .uniform(0) // Camera view-projection matrix
            .uniform(1) // Camera world position
            .storage(2, true) // Box transforms (from the updater)
            .storage(3, true) // Compacted visible indices
            .build();

        let render_bind_group = FrameBuffered::new(&gfx.device, render_layout, |i| {
            vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_pos_buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: transform_buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: visible_index_buffers[i].as_entire_binding(),
                },
            ]
        });

        let shader = gfx
            .device
            .create_shader_module(include_wgsl!("../../shaders/renderer/dynamic/ray_box.wgsl"));

        let render_pipeline = utils::pipeline::RenderPipelineBuilder::new(&gfx.device)
            .bind_group(&render_bind_group.layout)
            .vertex(&shader, &[])
            .fragment(&shader, &[Some(gfx.surface_config.format.into())])
            .with_depth_test()
            .with_frontface_culling()
            .build();

        Self {
            cull_pipeline,
            cull_bind_group,
            render_pipeline,
            render_bind_group,
            camera_pos_buffers,
            draw_args_buffers,
            cube_count,
        }
    }
}

impl Renderer for DynamicRenderer {
    fn render(&mut self, gfx: &mut Gfx, _world: &World, camera: &Camera, _size: PhysicalSize<u32>) {
        // The voxels live in raw world space, so the ray origin must be the
        // camera's world-space position (camera.position is in base-changed space).
        gfx.queue.write_buffer(
            &self.camera_pos_buffers[gfx.frame_index],
            0,
            bytemuck::bytes_of(&camera.world_position().extend(0.0)),
        );

        let draw_args = &self.draw_args_buffers[gfx.frame_index];

        let mut encoder = gfx.encoder.as_mut().unwrap();

        // Reset the survivor counter (instance_count, the second u32) before the
        // cull pass repopulates it. vertex_count and the offsets are left intact.
        encoder.clear_buffer(draw_args, 4, Some(4));

        {
            let mut cull_pass = utils::create_compute_pass(&mut encoder);
            cull_pass.set_pipeline(&self.cull_pipeline);
            cull_pass.set_bind_group(0, self.cull_bind_group.current(gfx.frame_index), &[]);
            cull_pass.dispatch_workgroups(
                u32::try_from(self.cube_count.div_ceil(64)).unwrap(),
                1,
                1,
            );
        }
        {
            let mut render_pass = utils::create_loading_render_pass(
                &mut encoder,
                gfx.surface_texture_view.as_ref().unwrap(),
                &gfx.depth_texture_view,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.render_bind_group.current(gfx.frame_index), &[]);
            render_pass.draw_indirect(draw_args, 0);
        }
    }
}
