#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
/// A simple vertex struct
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
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

/// The vertices to render a cube
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.5],
    }, // Front Face Top Left 0
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 0.5, 0.5],
    }, // Front Face Bottom Left 1
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.5, 0.5, 0.0],
    }, // Front Face Bottom Right 2
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    }, // Front Face Top Right 3
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.0, 0.5],
    }, // Back Face Top Left 4
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.5, 0.5],
    }, // Back Face Bottom Left 5
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.5, 0.5, 0.0],
    }, // Back Face Bottom Right 6
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    }, // Back Face Top Right 7
];

/// The indices to render a cube
pub const INDICES: &[u16] = &[
    0, 1, 2, // Front Face 1
    0, 2, 3, // Front Face 2
    4, 0, 3, // Top Face 1
    4, 3, 7, // Top Face 2
    7, 6, 5, // Back Face 1
    7, 5, 4, // Back Face 2
    1, 5, 6, // Bottom Face 1
    1, 6, 2, // Bottom Face 1
    3, 2, 6, // Right Face 1
    3, 6, 7, // Right Face 2
    4, 5, 1, // Left Face 1
    4, 1, 0, // Left Face 2
];
