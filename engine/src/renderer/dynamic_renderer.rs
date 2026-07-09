use super::Renderer;
use crate::camera::Camera;
use crate::constants::FRAMES_IN_FLIGHT;
use crate::core::bind_group::{BindGroupLayoutBuilder, FrameBuffered};
use crate::core::render_pass::RenderPassBuilder;
use crate::core::shaders::ShaderBuilder;
use crate::core::surface::Frame;
use crate::cube::Cube;
use crate::game::EngineContext;
use crate::gfx::Gfx;
use crate::utils::{self};

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
/// buffers produced by a separate [`Updater`](hello::updater::Updater); this
/// renderer only reads them and owns no per-box buffers of its own.
#[derive(Debug)]
pub struct DynamicRenderer {
    cull_pipeline: wgpu::ComputePipeline,
    cull_bind_group: FrameBuffered,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: FrameBuffered,
    // Per frame-in-flight: [vertex_count, instance_count, first_vertex, first_instance].
    // `instance_count` is reset to 0 and then filled by the cull pass each frame.
    draw_args_buffers: Vec<wgpu::Buffer>,
    cube_count: usize,
}

impl DynamicRenderer {
    /// `transform_buffers` are the per-frame hot transforms written by an
    /// [`Updater`](hello::updater::Updater); one per frame in flight, indexed by
    /// `gfx.frame_index`. The updater must run before this renderer each frame.
    pub fn init(
        gfx: &Gfx,
        camera: &Camera,
        cubes: &[Cube],
        transform_buffers: &[wgpu::Buffer],
    ) -> Self {
        let cube_count = cubes.len();

        let mut draw_args_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        let mut visible_index_buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);
        for i in 0..FRAMES_IN_FLIGHT {
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
        let cull_layout = BindGroupLayoutBuilder::new(&gfx.device)
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

        let cull_shader = ShaderBuilder::new(super::SHADER_DIR)
            .build_wgsl(&gfx.device, "package::box_cull")
            .expect("Failed to build shader");

        let cull_pipeline = gfx.create_compute_pipeline(
            "ETIB Dynamic Cull",
            Some(&cull_bind_group.layout),
            0,
            &cull_shader,
        );

        // Render pass: camera + the transforms plus the survivor list, which the
        // vertex shader indexes by `instance_index` to recover the real box.
        let render_layout = BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::VERTEX_FRAGMENT)
            .uniform(0) // Camera struct
            .storage(1, true) // Box transforms (from the updater)
            .storage(2, true) // Compacted visible indices
            .build();

        let render_bind_group = FrameBuffered::new(&gfx.device, render_layout, |i| {
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
                    resource: visible_index_buffers[i].as_entire_binding(),
                },
            ]
        });

        let shader = ShaderBuilder::new(super::SHADER_DIR)
            .build_wgsl(&gfx.device, "package::ray_box")
            .expect("Failed to build shader");

        let render_pipeline = crate::core::pipeline::RenderPipelineBuilder::new(&gfx.device)
            .bind_group(&render_bind_group.layout)
            .vertex(&shader, &[])
            .fragment(&shader, &[Some(gfx.surface.config.format.into())])
            .with_depth_test()
            .with_frontface_culling()
            .build();

        Self {
            cull_pipeline,
            cull_bind_group,
            render_pipeline,
            render_bind_group,
            draw_args_buffers,
            cube_count,
        }
    }
}

impl Renderer for DynamicRenderer {
    fn render(&mut self, ctx: &mut EngineContext, frame: &mut Frame) {
        let gfx = &mut ctx.gfx;

        let draw_args = &self.draw_args_buffers[gfx.frame_index];

        let encoder = &mut frame.encoder;

        // Reset the survivor counter (instance_count, the second u32) before the
        // cull pass repopulates it. vertex_count and the offsets are left intact.
        encoder.clear_buffer(draw_args, 4, Some(4));

        {
            let mut cull_pass = gfx.create_compute_pass("dynamic_renderer cull pass", encoder);
            cull_pass.set_pipeline(&self.cull_pipeline);
            cull_pass.set_bind_group(0, self.cull_bind_group.current(gfx.frame_index), &[]);
            cull_pass.dispatch_workgroups(
                u32::try_from(self.cube_count.div_ceil(64)).unwrap(),
                1,
                1,
            );
        }
        {
            let mut render_pass = RenderPassBuilder::new()
                .target(&frame.view, wgpu::LoadOp::Load, wgpu::StoreOp::Store)
                .depth_stencil_view(&gfx.depth.view)
                .depth_ops(wgpu::LoadOp::Load, wgpu::StoreOp::Store)
                .build(encoder);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.render_bind_group.current(gfx.frame_index), &[]);
            render_pass.draw_indirect(draw_args, 0);
        }
    }
}
