/// A simple vertex struct representing a 3D point with color and normal
///
/// This structure contains the basic data for a single vertex: its 3D position,
/// RGB color values, and normal vector. The layout is compatible with GPU vertex buffers.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// The 3D position of the vertex (x, y, z)
    pub position: [f32; 3],
    /// The RGB color of the vertex (r, g, b) with values in range [0.0, 1.0]
    pub color: [f32; 3],
    /// The normal vector of the vertex (nx, ny, nz)
    pub normal: [f32; 3],
}

impl Vertex {
    /// Returns a buffer layout
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: (size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_desc() {
        let desc = Vertex::desc();
        assert_eq!(desc.array_stride, std::mem::size_of::<Vertex>() as u64);
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(desc.attributes.len(), 3);
        
        assert_eq!(desc.attributes[0].shader_location, 0);
        assert_eq!(desc.attributes[1].shader_location, 1);
        assert_eq!(desc.attributes[2].shader_location, 2);
    }
}
