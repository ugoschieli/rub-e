/// A simple vertex struct representing a 3D point with color
///
/// This structure contains the basic data for a single vertex: its 3D position
/// and RGB color values. The layout is compatible with GPU vertex buffers.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// The 3D position of the vertex (x, y, z)
    pub position: [f32; 3],
    /// The RGB color of the vertex (r, g, b) with values in range [0.0, 1.0]
    pub color: [f32; 3],
}

impl Vertex {
    /// Returns a buffer layout
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
