use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// An axis-aligned cube in the raytracing world.
#[derive(Debug, Clone, Copy)]
pub struct Cube {
    /// Center position in world space.
    pub position: Vec3,
    /// Half-extent of the cube (i.e. the cube spans ±size around `position`).
    pub size: f32,
    /// Surface material.
    pub material: Material,
}

/// Surface material for a raytraced cube.
#[derive(Debug, Clone, Copy)]
pub enum Material {
    /// Diffuse Lambertian surface with the given albedo color.
    Lambertian(Vec3),
    /// Metallic surface with albedo and roughness (0.0 = mirror, 1.0 = fully rough).
    Metal(Vec3, f32),
    /// Dielectric (glass-like) surface with the given refraction index.
    Dielectric(f32),
    /// Emissive (light-emitting) surface with the given emission color.
    Emissive(Vec3),
}

/// GPU-compatible representation of a cube for the raytracing compute shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CubeUniform {
    /// GPU-compatible material data.
    pub mat: MaterialUniform,
    /// Center position in world space.
    pub center: Vec3,
    /// Half-extent of the cube.
    pub size: f32,
}

/// GPU-compatible representation of a material for the raytracing compute shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    /// Material type discriminant (see `MAT_*` constants).
    pub mat_type: u32,
    /// Surface roughness (used for `Metal`).
    pub roughness: f32,
    /// Index of refraction (used for `Dielectric`).
    pub refraction_index: f32,
    _pad: u32,
    /// Albedo or emission color.
    pub albedo: Vec3,
    _pad1: u32,
}

/// Material type constant for Lambertian surfaces.
pub const MAT_LAMBERTIAN: u32 = 0;
/// Material type constant for Metal surfaces.
pub const MAT_METAL: u32 = 1;
/// Material type constant for Dielectric surfaces.
pub const MAT_DIELECTRIC: u32 = 2;
/// Material type constant for Emissive surfaces.
pub const MAT_EMISSIVE: u32 = 3;

/// A collection of cubes that make up the raytraced scene.
pub struct World {
    /// All cubes in the scene.
    pub cubes: Vec<Cube>,
}

impl Cube {
    /// Create a new cube at `position` with the given half-`size` and `material`.
    pub fn new(position: Vec3, size: f32, material: Material) -> Self {
        Self {
            position,
            size,
            material,
        }
    }

    /// Convert to the GPU-compatible uniform representation.
    pub fn to_uniform(&self) -> CubeUniform {
        CubeUniform {
            mat: self.material.to_uniform(),
            center: self.position,
            size: self.size,
        }
    }
}

impl World {
    /// Create a world from a slice of cubes.
    pub fn new(cubes: &[Cube]) -> Self {
        Self {
            cubes: cubes.to_vec(),
        }
    }

    /// Convert all cubes to their GPU-compatible uniform representation.
    pub fn to_uniform(&self) -> Vec<CubeUniform> {
        self.cubes.iter().map(|cube| cube.to_uniform()).collect()
    }

    /// Return the indices of all emissive cubes (used to importance-sample lights).
    pub fn get_light_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();
        for (i, cube) in self.cubes.iter().enumerate() {
            if let Material::Emissive(_) = cube.material {
                indices.push(i as u32);
            }
        }
        indices
    }
}

impl Material {
    /// Convert to the GPU-compatible uniform representation.
    pub fn to_uniform(&self) -> MaterialUniform {
        match self {
            Material::Lambertian(albedo) => MaterialUniform {
                mat_type: MAT_LAMBERTIAN,
                roughness: 0.0,
                refraction_index: 0.0,
                _pad: 0,
                albedo: *albedo,
                _pad1: 0,
            },
            Material::Metal(albedo, roughness) => MaterialUniform {
                mat_type: MAT_METAL,
                roughness: *roughness,
                refraction_index: 0.0,
                _pad: 0,
                albedo: *albedo,
                _pad1: 0,
            },
            Material::Dielectric(refraction_index) => MaterialUniform {
                mat_type: MAT_DIELECTRIC,
                roughness: 0.0,
                refraction_index: *refraction_index,
                _pad: 0,
                albedo: Vec3::ZERO,
                _pad1: 0,
            },
            Material::Emissive(albedo) => MaterialUniform {
                mat_type: MAT_EMISSIVE,
                roughness: 0.0,
                refraction_index: 0.0,
                _pad: 0,
                albedo: *albedo,
                _pad1: 0,
            },
        }
    }
}

/// A vertex with position and normal used for G-buffer rasterization.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    /// 3D position of the vertex.
    pub position: [f32; 3],
    /// Surface normal at this vertex.
    pub normal: [f32; 3],
}

impl Vertex {
    /// Returns the wgpu vertex buffer layout descriptor for this vertex type.
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

/// Vertices for a unit cube centered at the origin, with per-face normals.
pub const CUBE_VERTICES: &[Vertex] = &[
    // Front face (Z = 1)
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
    // Back face (Z = -1)
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
    },
    // Top face (Y = 1)
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [0.0, 1.0, 0.0],
    },
    // Bottom face (Y = -1)
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
    },
    // Right face (X = 1)
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
    },
    // Left face (X = -1)
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [-1.0, 0.0, 0.0],
    },
];

/// Index buffer for the unit cube, two triangles per face in CCW order.
pub const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // front
    4, 5, 6, 4, 6, 7, // back
    8, 9, 10, 8, 10, 11, // top
    12, 13, 14, 12, 14, 15, // bottom
    16, 17, 18, 16, 18, 19, // right
    20, 21, 22, 20, 22, 23, // left
];
