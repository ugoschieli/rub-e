use cgmath::Zero;

use crate::Vertex;

/// Represents a cube instance with transformation and color information
pub struct Cube {
    /// The 4x4 transformation matrix for positioning, rotating, and scaling the cube
    pub model: cgmath::Matrix4<f32>,
    /// The RGBA color vector for the cube
    pub color: cgmath::Vector4<f32>,
}

/// Raw GPU-compatible representation of a cube instance
///
/// This struct is laid out in a way that can be directly uploaded to the GPU
/// for instanced rendering. It contains the model transformation matrix and color.
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
    /// Create a new cube instance with default (zero) transformation and color
    ///
    /// # Returns
    /// A new `Cube` with identity matrix and black color
    pub fn new() -> Self {
        Self {
            model: cgmath::Matrix4::zero(),
            color: cgmath::Vector4::zero(),
        }
    }

    /// Convert this Cube into its raw GPU-compatible representation
    ///
    /// # Returns
    /// A `CubeRaw` instance ready for GPU upload
    pub fn into_raw(self) -> CubeRaw {
        self.into()
    }

    /// Create an instance buffer for cube rendering
    ///
    /// This function is currently a placeholder for future instanced rendering implementation.
    pub fn create_instance_buffer() {}

    /// Get the vertex buffer layout descriptor for cube instances
    ///
    /// This describes how cube instance data is laid out in GPU memory for the vertex shader.
    /// The layout includes the model transformation matrix (4x Vec4) distributed across
    /// shader locations 2-5.
    ///
    /// # Returns
    /// A `wgpu::VertexBufferLayout` describing the instance data structure
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

/// The vertices to render a cube with normals for lighting
pub const VERTICES: &[Vertex] = &[
    // Front face (Z+)
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.5],
        normal: [0.0, 0.0, 1.0],
    }, // 0
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    }, // 1
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.5, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
    }, // 2
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    }, // 3
    // Back face (Z-)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.0, 0.5],
        normal: [0.0, 0.0, -1.0],
    }, // 4
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.5, 0.5],
        normal: [0.0, 0.0, -1.0],
    }, // 5
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.5, 0.5, 0.0],
        normal: [0.0, 0.0, -1.0],
    }, // 6
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        normal: [0.0, 0.0, -1.0],
    }, // 7
    // Top face (Y+) - duplicate vertices with different normals
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.0, 0.5],
        normal: [0.0, 1.0, 0.0],
    }, // 8
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.5],
        normal: [0.0, 1.0, 0.0],
    }, // 9
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    }, // 10
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
    }, // 11
    // Bottom face (Y-)
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    }, // 12
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    }, // 13
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.5, 0.5, 0.0],
        normal: [0.0, -1.0, 0.0],
    }, // 14
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.5, 0.5, 0.0],
        normal: [0.0, -1.0, 0.0],
    }, // 15
    // Right face (X+)
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [1.0, 0.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    }, // 16
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.5, 0.5, 0.0],
        normal: [1.0, 0.0, 0.0],
    }, // 17
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.5, 0.5, 0.0],
        normal: [1.0, 0.0, 0.0],
    }, // 18
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [1.0, 0.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    }, // 19
    // Left face (X-)
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.5, 0.0, 0.5],
        normal: [-1.0, 0.0, 0.0],
    }, // 20
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.0, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    }, // 21
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.0, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    }, // 22
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.5, 0.0, 0.5],
        normal: [-1.0, 0.0, 0.0],
    }, // 23
];

/// The indices to render a cube with proper normals and CCW winding
pub const INDICES: &[u16] = &[
    // Front Face (Z+) - CCW from outside: TL -> BL -> BR -> TR
    0, 1, 2, // Triangle 1: TL, BL, BR
    0, 2, 3, // Triangle 2: TL, BR, TR
    // Back Face (Z-) - CCW from outside: TR -> BR -> BL -> TL
    4, 7, 6, // Triangle 1: TL, TR, BR
    4, 6, 5, // Triangle 2: TL, BR, BL
    // Top Face (Y+) - CCW from above: BL -> FL -> FR -> BR
    8, 9, 10, // Triangle 1
    8, 10, 11, // Triangle 2
    // Bottom Face (Y-) - CCW from below: FL -> BL -> BR -> FR
    12, 13, 14, // Triangle 1
    12, 14, 15, // Triangle 2
    // Right Face (X+) - CCW from outside: TF -> BF -> BB -> TB
    16, 17, 18, // Triangle 1
    16, 18, 19, // Triangle 2
    // Left Face (X-) - CCW from outside: TB -> BB -> BF -> TF
    20, 21, 22, // Triangle 1
    20, 22, 23, // Triangle 2
];
