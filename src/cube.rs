use cgmath::Zero;

use crate::Vertex;

pub struct Cube {
    model: cgmath::Matrix4<f32>,
    color: cgmath::Vector4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Into<CubeRaw> for Cube {
    fn into(self) -> CubeRaw {
        CubeRaw {
            model: self.model.into(),
            color: self.color.into(),
        }
    }
}

impl Cube {
    pub fn new() -> Self {
        Self {
            model: cgmath::Matrix4::zero(),
            color: cgmath::Vector4::zero(),
        }
    }

    pub fn create_instance_buffer() {}

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CubeRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    format: wgpu::VertexFormat::Float32x4,
                    shader_location: 5,
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
