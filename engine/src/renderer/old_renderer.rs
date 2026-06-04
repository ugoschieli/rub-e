use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Mat4, Vec3};
use winit::dpi::PhysicalSize;

use crate::{
    camera::Camera,
    chunk::World,
    constants::{CUBE_NUMBER, FRAMES_IN_FLIGHT},
    frame_buffer::FrameBuffers,
    gfx::Gfx,
    renderer::Renderer,
    utils,
};

#[derive(Debug)]
pub struct OldRenderer {
    pub cull_pipeline: wgpu::ComputePipeline,
    pub mesh_pipeline: wgpu::ComputePipeline,
    pub render_pipeline: wgpu::RenderPipeline,
    pub frame_buffers: [FrameBuffers; FRAMES_IN_FLIGHT],
}

impl OldRenderer {
    pub fn init(gfx: &crate::gfx::Gfx, world: &World) -> Self {
        let face_matrices = &[
            Mat4::from_translation(Vec3::new(0.5, 0., 0.)) * Mat4::from_rotation_y(FRAC_PI_2),
            Mat4::from_translation(Vec3::new(-0.5, 0., 0.)) * Mat4::from_rotation_y(-FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., 0.5, 0.)) * Mat4::from_rotation_x(-FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., -0.5, 0.)) * Mat4::from_rotation_x(FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., 0., 0.5)) * Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0., 0., -0.5)) * Mat4::from_rotation_y(-PI),
        ];

        let face_matrices_buffer = utils::create_buffer_init(
            &gfx.device,
            "ETIB Face Rotation Matrices Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            face_matrices,
        );

        let frame_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                FrameBuffers::new(
                    &gfx.device,
                    CUBE_NUMBER,
                    world.chunk_count,
                    &face_matrices_buffer,
                    &world.chunk_meta_buffer,
                    &world.voxel_buffer,
                )
            })
            .collect::<Vec<FrameBuffers>>();

        let cull_pipeline = utils::create_compute_pipeline(
            &gfx.device,
            "Frustum Culling",
            &frame_buffers[0].cull_bind_group_layout,
            "./shaders/cull.wgsl",
        );

        let mesh_pipeline = utils::create_compute_pipeline(
            &gfx.device,
            "Mesh Generation",
            &frame_buffers[0].mesh_bind_group_layout,
            "./shaders/mesh.wgsl",
        );

        let render_pipeline = utils::create_render_pipeline(
            &gfx.device,
            &gfx.surface_config,
            &frame_buffers[0].render_bind_group_layout,
        );

        Self {
            cull_pipeline,
            mesh_pipeline,
            render_pipeline,
            frame_buffers: frame_buffers.try_into().unwrap(),
        }
    }
}

impl Renderer for OldRenderer {
    fn render(&mut self, gfx: &mut Gfx, world: &World, camera: &Camera, size: PhysicalSize<u32>) {
        let frame_buffer = &self.frame_buffers[gfx.frame_index];

        gfx.queue.write_buffer(
            &frame_buffer.camera_buffer,
            0,
            bytemuck::bytes_of(&camera.matrix(size)),
        );

        let mut encoder = gfx.encoder.as_mut().unwrap();
        // Reset the cull counter and the draw vertex_count before the compute passes.
        encoder.clear_buffer(&frame_buffer.visible_chunks_buffer, 0, Some(4));
        encoder.clear_buffer(&frame_buffer.draw_indirect_buffer, 0, Some(4));
        {
            let mut cull_pass = utils::create_compute_pass(&mut encoder);
            cull_pass.set_pipeline(&self.cull_pipeline);
            cull_pass.set_bind_group(0, &frame_buffer.cull_bind_group, &[]);
            cull_pass.dispatch_workgroups(
                u32::try_from(world.chunk_count.div_ceil(64)).unwrap(),
                1,
                1,
            );
        }
        {
            let mut mesh_pass = utils::create_compute_pass(&mut encoder);
            mesh_pass.set_pipeline(&self.mesh_pipeline);
            mesh_pass.set_bind_group(0, &frame_buffer.mesh_bind_group, &[]);
            mesh_pass.dispatch_workgroups(u32::try_from(world.chunk_count).unwrap(), 1, 1);
        }
        {
            let mut render_pass = utils::create_render_pass(
                &mut encoder,
                gfx.surface_texture_view.as_ref().unwrap(),
                &gfx.depth_texture_view,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &frame_buffer.render_bind_group, &[]);
            render_pass.draw_indirect(&frame_buffer.draw_indirect_buffer, 0);
        }
    }
}
