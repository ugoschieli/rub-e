use crate::cube::CubeGpu;
use crate::utils;
use glam::Mat4;

#[derive(Debug)]
pub struct FrameBuffers {
    pub camera_buffer: wgpu::Buffer,
    pub face_buffer: wgpu::Buffer,
    pub compute_bind_group_layout: wgpu::BindGroupLayout,
    pub render_bind_group_layout: wgpu::BindGroupLayout,
    pub compute_bind_group: wgpu::BindGroup,
    pub render_bind_group: wgpu::BindGroup,
}

impl FrameBuffers {
    pub fn new(
        device: &wgpu::Device,
        cubes: &[CubeGpu],
        cubes_buffer: &wgpu::Buffer,
        face_matrices_buffer: &wgpu::Buffer,
    ) -> Self {
        let camera_buffer = utils::create_buffer_init(
            device,
            "ETIB Camera Buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[Mat4::ZERO],
        );

        let face_buffer = utils::create_buffer(
            device,
            "EITB Faces Buffer",
            (cubes.len() * 6 * 48) as u64, // 6 Faces max foreach cubes, a Face is 48 bytes (2 vec4<f32> + u32 + padding)
            wgpu::BufferUsages::STORAGE,
        );

        let (compute_bind_group, compute_bind_group_layout) =
            utils::create_compute_bind_group(device, cubes_buffer, &face_buffer);

        let (render_bind_group, render_bind_group_layout) =
            utils::create_bind_group(device, &camera_buffer, &face_buffer, face_matrices_buffer);

        Self {
            camera_buffer,
            face_buffer,
            compute_bind_group_layout,
            render_bind_group_layout,
            compute_bind_group,
            render_bind_group,
        }
    }
}
