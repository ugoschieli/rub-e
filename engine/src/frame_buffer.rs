use crate::utils;
use glam::Mat4;

#[derive(Debug, Clone)]
pub struct FrameBuffers {
    pub camera_buffer: wgpu::Buffer,
    pub faces_buffer: wgpu::Buffer,
    // count (u32) + visible chunk indices (u32[]); reset to 0 each frame before culling.
    pub visible_chunks_buffer: wgpu::Buffer,
    // First 16 bytes are DrawIndirectArgs: vertex_count (atomic), instance_count=1, first_vertex=0, first_instance=0.
    // vertex_count is reset to 0 each frame before the mesh pass.
    pub draw_indirect_buffer: wgpu::Buffer,
    pub cull_bind_group_layout: wgpu::BindGroupLayout,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,
    pub render_bind_group_layout: wgpu::BindGroupLayout,
    pub cull_bind_group: wgpu::BindGroup,
    pub mesh_bind_group: wgpu::BindGroup,
    pub render_bind_group: wgpu::BindGroup,
}

impl FrameBuffers {
    pub fn new(
        device: &wgpu::Device,
        max_voxels: usize,
        chunk_count: usize,
        face_matrices_buffer: &wgpu::Buffer,
        chunk_meta_buffer: &wgpu::Buffer,
        voxel_buffer: &wgpu::Buffer,
    ) -> Self {
        let camera_buffer = utils::create_buffer_init(
            device,
            "ETIB Camera Buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[Mat4::ZERO],
        );

        let faces_buffer = utils::create_buffer(
            device,
            "ETIB Faces Buffer",
            max_voxels as u64 * 6 * 48,
            wgpu::BufferUsages::STORAGE,
        );

        // Layout: [count: u32][index_0: u32]...[index_N: u32]
        let visible_chunks_buffer = utils::create_buffer(
            device,
            "ETIB Visible Chunks Buffer",
            4 + chunk_count as u64 * 4,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        );

        // [vertex_count=0, instance_count=1, first_vertex=0, first_instance=0]
        let draw_indirect_buffer = utils::create_buffer_init(
            device,
            "ETIB Draw Indirect Buffer",
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            &[0u32, 1u32, 0u32, 0u32],
        );

        let (cull_bind_group, cull_bind_group_layout) = utils::create_cull_bind_group(
            device,
            &camera_buffer,
            chunk_meta_buffer,
            &visible_chunks_buffer,
        );

        let (mesh_bind_group, mesh_bind_group_layout) = utils::create_mesh_bind_group(
            device,
            &visible_chunks_buffer,
            chunk_meta_buffer,
            voxel_buffer,
            &draw_indirect_buffer,
            &faces_buffer,
        );

        let (render_bind_group, render_bind_group_layout) =
            utils::create_bind_group(device, &camera_buffer, &faces_buffer, face_matrices_buffer);

        Self {
            camera_buffer,
            faces_buffer,
            visible_chunks_buffer,
            draw_indirect_buffer,
            cull_bind_group_layout,
            mesh_bind_group_layout,
            render_bind_group_layout,
            cull_bind_group,
            mesh_bind_group,
            render_bind_group,
        }
    }
}
