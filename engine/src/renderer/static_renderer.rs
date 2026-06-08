use std::f32::consts::{FRAC_PI_2, PI};

use bytemuck::{Pod, Zeroable};
use glam::{uvec3, Mat4, UVec3, Vec3};
use wgpu::include_wgsl;
use winit::dpi::PhysicalSize;

use crate::chunk::VoxelGpu;
use crate::constants::CHUNK_SIZE_3;
use crate::mesher::{chunk_index, mesh_chunk};
use crate::{
    camera::Camera,
    chunk::World,
    gfx::Gfx,
    renderer::Renderer,
    utils::{self, bindgroup::FrameBuffered},
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct FaceGpu {
    position: Vec3,
    width: u32,
    height: u32,
    direction: u32,
    color: u32,
    _pad: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub position: Vec3,
    pub width: u32,
    pub height: u32,
    pub direction: FaceDirection,
    pub color: UVec3,
}

impl Face {
    pub fn new(
        position: Vec3,
        width: u32,
        height: u32,
        direction: FaceDirection,
        color: UVec3,
    ) -> Self {
        Self {
            position,
            width,
            height,
            direction,
            color,
        }
    }

    pub fn to_gpu(&self) -> FaceGpu {
        FaceGpu {
            position: self.position,
            width: self.width,
            height: self.height,
            direction: self.direction.to_gpu(),
            color: pack_color(self.color),
            _pad: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FaceDirection {
    Right,  // +X = 0
    Left,   // -X = 1
    Forward,   // +Y = 2
    Backward,  // -Y = 3
    Up,    // +Z = 4
    Down, // -Z = 5
}

impl FaceDirection {
    pub fn to_gpu(&self) -> u32 {
        match self {
            FaceDirection::Right => 0,
            FaceDirection::Left => 1,
            FaceDirection::Forward => 2,
            FaceDirection::Backward => 3,
            FaceDirection::Up => 4,
            FaceDirection::Down => 5,
        }
    }
}

pub fn pack_color(color: UVec3) -> u32 {
    color.x | (color.y << 10) | (color.z << 20) // packed color (10 bit for each channel)
}

pub fn unpack_color(color: u32) -> UVec3 {
    let r = color & 1023;
    let g = (color >> 10) & 1023;
    let b = (color >> 20) & 1023;

    UVec3::new(r, g, b)
}

#[derive(Debug)]
pub struct StaticRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: FrameBuffered,
    faces_len: usize,
}

impl StaticRenderer {
    pub fn init(gfx: &Gfx, world: &World, camera: &Camera) -> Self {

        let mut chunk = [VoxelGpu { color: 0 }; CHUNK_SIZE_3];

        chunk[chunk_index(uvec3(0, 0, 0))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(1, 0, 0))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(2, 0, 0))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(0, 0, 1))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(1, 0, 1))] = VoxelGpu {color: 2u32.pow(20) - 1};
        chunk[chunk_index(uvec3(2, 0, 1))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(0, 0, 2))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(1, 0, 2))] = VoxelGpu {color: 1023};
        chunk[chunk_index(uvec3(2, 0, 2))] = VoxelGpu {color: 1023};

        chunk[chunk_index(uvec3(0, 1, 0))] = VoxelGpu {color: 1023};

        let faces = mesh_chunk(&chunk).iter().map(|face| face.to_gpu()).collect::<Vec<FaceGpu>>();

        // The flipped faces (Left, Forward, Down) are pure rotations: their
        // rotation mirrors an extent axis, so a constant translation can only
        // place a 1-wide quad correctly. The size-dependent offset is applied
        // to Face.position in the mesher instead (see mesh_chunk).
        let face_matrices = &[
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)) * Mat4::from_rotation_z(FRAC_PI_2), // Right Face
            Mat4::from_rotation_z(-FRAC_PI_2), // Left Face
            Mat4::from_rotation_z(PI), // Back Face
            Mat4::IDENTITY, // Front Face
            Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)) * Mat4::from_rotation_x(-FRAC_PI_2), // Top Face
            Mat4::from_rotation_x(FRAC_PI_2), // Bottom Face
        ];

        let face_matrices_buffer = utils::create_buffer_init(
            &gfx.device,
            "ETIB Face Rotation Matrices Buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            face_matrices,
        );

        let face_buffer = utils::create_buffer_init(
            &gfx.device,
            "ETIB Face Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            &faces,
        );

        let layout = utils::bindgroup::BindGroupLayoutBuilder::new(&gfx.device)
            .visibility(wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX_FRAGMENT)
            .uniform(0) // Camera
            .uniform(1) // Face Matrices
            .storage(2, true)
            .build();

        let bind_group = FrameBuffered::new(&gfx.device, layout, |i| {
            vec![
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffers[i].as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: face_matrices_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: face_buffer.as_entire_binding(),
                },
            ]
        });

        let shader = gfx
            .device
            .create_shader_module(include_wgsl!("../../shaders/renderer/static/face_draw.wgsl"));

        let render_pipeline = utils::pipeline::RenderPipelineBuilder::new(&gfx.device)
            .bind_group(&bind_group.layout)
            .vertex(&shader, &[])
            .fragment(&shader, &[Some(gfx.surface_config.format.into())])
            .with_depth_test()
            .with_backface_culling()
            .build();

        Self {
            render_pipeline,
            bind_group,
            faces_len: faces.len(),
        }
    }
}

impl Renderer for StaticRenderer {
    fn render(&mut self, gfx: &mut Gfx, world: &World, camera: &Camera, size: PhysicalSize<u32>) {
        let mut encoder = gfx.encoder.as_mut().unwrap();
        {
            let mut render_pass = utils::create_render_pass(
                &mut encoder,
                gfx.surface_texture_view.as_ref().unwrap(),
                &gfx.depth_texture_view,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.bind_group.current(gfx.frame_index), &[]);
            render_pass.draw(0..(6 * self.faces_len as u32), 0..1);
            // render_pass.draw_indirect(&frame_buffer.draw_indirect_buffer, 0);
        }
    }
}
