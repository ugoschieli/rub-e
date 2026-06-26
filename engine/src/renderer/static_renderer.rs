use std::f32::consts::{FRAC_PI_2, PI};

use super::Renderer;
use crate::camera::Camera;
use crate::constants::CHUNK_SIZE_3;
use crate::core::bind_group::FrameBuffered;
use crate::core::render_pass::RenderPassBuilder;
use crate::core::shaders::ShaderBuilder;
use crate::core::surface::Frame;
use crate::cube::VoxelGpu;
use crate::game::EngineContext;
use crate::gfx::Gfx;
use crate::mesher::{Face, FaceGpu, chunk_index, mesh_chunk};
use crate::utils;
use glam::{Mat4, Vec3, uvec3};

#[derive(Debug)]
pub struct StaticRenderer {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: FrameBuffered,
    faces_len: usize,
}

impl StaticRenderer {
    pub fn init(gfx: &Gfx, camera: &Camera) -> Self {
        // Heap-allocate the chunk: as a stack array it's ~931 KB (62^3 * 4 B),
        // which overflows Windows' 1 MB main-thread stack (macOS gets 8 MB).
        let mut chunk: Box<[VoxelGpu; CHUNK_SIZE_3]> = vec![VoxelGpu { color: 0 }; CHUNK_SIZE_3]
            .into_boxed_slice()
            .try_into()
            .unwrap();

        chunk[chunk_index(uvec3(0, 0, 10))] = VoxelGpu { color: 1023 };
        // chunk[chunk_index(uvec3(1, 0, 0))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(2, 0, 0))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(0, 0, 1))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(1, 0, 1))] = VoxelGpu {color: 2u32.pow(20) - 1};
        // chunk[chunk_index(uvec3(2, 0, 1))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(0, 0, 2))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(1, 0, 2))] = VoxelGpu {color: 1023};
        // chunk[chunk_index(uvec3(2, 0, 2))] = VoxelGpu {color: 1023};
        //
        // chunk[chunk_index(uvec3(0, 1, 0))] = VoxelGpu {color: 1023};

        let faces = mesh_chunk(&chunk)
            .iter()
            .map(Face::to_gpu)
            .collect::<Vec<FaceGpu>>();

        // The flipped faces (Left, Forward, Down) are pure rotations: their
        // rotation mirrors an extent axis, so a constant translation can only
        // place a 1-wide quad correctly. The size-dependent offset is applied
        // to Face.position in the mesher instead (see mesh_chunk).
        let face_matrices = &[
            Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)) * Mat4::from_rotation_z(FRAC_PI_2), // Right Face
            Mat4::from_rotation_z(-FRAC_PI_2), // Left Face
            Mat4::from_rotation_z(PI),         // Back Face
            Mat4::IDENTITY,                    // Front Face
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

        let layout = crate::core::bind_group::BindGroupLayoutBuilder::new(&gfx.device)
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

        let shader = ShaderBuilder::new(super::SHADER_DIR)
            .build_wgsl(&gfx.device, "package::face_draw")
            .unwrap();

        let render_pipeline = crate::core::pipeline::RenderPipelineBuilder::new(&gfx.device)
            .bind_group(&bind_group.layout)
            .vertex(&shader, &[])
            .fragment(&shader, &[Some(gfx.surface.config.format.into())])
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
    fn render(&mut self, ctx: &mut EngineContext, frame: &mut Frame) {
        let encoder = &mut frame.encoder;
        {
            let mut render_pass = RenderPassBuilder::new()
                .target(
                    &frame.view,
                    wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    wgpu::StoreOp::Store,
                )
                .depth_stencil_view(&ctx.gfx.depth.view)
                .depth_ops(wgpu::LoadOp::Clear(1.0), wgpu::StoreOp::Store)
                .build(encoder);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.bind_group.current(ctx.gfx.frame_index), &[]);
            render_pass.draw(0..(6 * u32::try_from(self.faces_len).unwrap()), 0..1);
            // render_pass.draw_indirect(&frame_buffer.draw_indirect_buffer, 0);
        }
    }
}
