use crate::app::{App, CUBE_NUMBER};
use crate::camera::Camera;
use crate::chunk::World;
use crate::frame_buffer::FrameBuffers;
use crate::time::Time;
use crate::utils;
use glam::{Mat4, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub const FRAMES_IN_FLIGHT: usize = 2;

#[derive(Debug)]
pub struct Gfx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    cull_pipeline: wgpu::ComputePipeline,
    mesh_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture_view: wgpu::TextureView,
    frame_buffers: [FrameBuffers; FRAMES_IN_FLIGHT],
    submission_indices: [Option<wgpu::SubmissionIndex>; FRAMES_IN_FLIGHT],
}

impl Gfx {
    pub fn new(event_loop: &ActiveEventLoop, window: Arc<Window>, app: &mut App) -> Self {
        let size = window.inner_size();

        let instance = utils::create_instance(event_loop);
        let adapter = utils::create_adapter(&instance);
        let (device, queue) = utils::create_device(&adapter);
        let (surface, surface_config) =
            utils::create_surface(&instance, &adapter, &device, window, size);

        app.world = Some(World::new(&device, &app.cubes));

        let face_matrices = &[
            Mat4::from_translation(Vec3::new(0.5, 0., 0.)) * Mat4::from_rotation_y(FRAC_PI_2),
            Mat4::from_translation(Vec3::new(-0.5, 0., 0.)) * Mat4::from_rotation_y(-FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., 0.5, 0.)) * Mat4::from_rotation_x(-FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., -0.5, 0.)) * Mat4::from_rotation_x(FRAC_PI_2),
            Mat4::from_translation(Vec3::new(0., 0., 0.5)) * Mat4::IDENTITY,
            Mat4::from_translation(Vec3::new(0., 0., -0.5)) * Mat4::from_rotation_y(-PI),
        ];

        let face_matrices_buffer = utils::create_buffer_init(
            &device,
            "ETIB Face Rotation Matrices Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            face_matrices,
        );

        let frame_buffers = (0..FRAMES_IN_FLIGHT)
            .map(|_| {
                FrameBuffers::new(
                    &device,
                    CUBE_NUMBER,
                    app.world.as_ref().unwrap().chunk_count,
                    &face_matrices_buffer,
                    &app.world.as_ref().unwrap().chunk_meta_buffer,
                    &app.world.as_ref().unwrap().voxel_buffer,
                )
            })
            .collect::<Vec<FrameBuffers>>();

        let cull_pipeline = utils::create_compute_pipeline(
            &device,
            "Frustum Culling",
            &frame_buffers[0].cull_bind_group_layout,
            "./shaders/cull.wgsl",
        );

        let mesh_pipeline = utils::create_compute_pipeline(
            &device,
            "Mesh Generation",
            &frame_buffers[0].mesh_bind_group_layout,
            "./shaders/mesh.wgsl",
        );

        let render_pipeline = utils::create_render_pipeline(
            &device,
            &surface_config,
            &frame_buffers[0].render_bind_group_layout,
        );

        let (_depth_texture, depth_texture_view) = utils::create_depth_texture(&device, size);

        Self {
            device,
            queue,
            surface,
            cull_pipeline,
            mesh_pipeline,
            render_pipeline,
            depth_texture_view,
            frame_buffers: frame_buffers.try_into().unwrap(),
            submission_indices: [const { None }; FRAMES_IN_FLIGHT],
        }
    }

    pub fn render(&mut self, camera: &Camera, size: PhysicalSize<u32>, time: &Time, world: &World) {
        let frame_index = time.frame_number % FRAMES_IN_FLIGHT;

        if let Some(idx) = self.submission_indices[frame_index].take() {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(idx),
                    timeout: None,
                })
                .unwrap();
        }

        let frame_buffer = &self.frame_buffers[frame_index];

        let Some((current_surface_texture, current_surface_texture_view)) =
            utils::get_current_surface_texture(&self.surface)
        else {
            return;
        };

        self.queue.write_buffer(
            &frame_buffer.camera_buffer,
            0,
            bytemuck::bytes_of(&camera.matrix(size)),
        );

        let mut encoder = utils::create_encoder(&self.device);
        // Reset the cull counter and the draw vertex_count before the compute passes.
        encoder.clear_buffer(&frame_buffer.visible_chunks_buffer, 0, Some(4));
        encoder.clear_buffer(&frame_buffer.draw_indirect_buffer, 0, Some(4));
        {
            let mut cull_pass = utils::create_compute_pass(&mut encoder);
            cull_pass.set_pipeline(&self.cull_pipeline);
            cull_pass.set_bind_group(0, &frame_buffer.cull_bind_group, &[]);
            cull_pass.dispatch_workgroups(
                u32::try_from(world.chunk_count.div_ceil(64)).unwrap(),
                1,
                1,
            );
        }
        {
            let mut mesh_pass = utils::create_compute_pass(&mut encoder);
            mesh_pass.set_pipeline(&self.mesh_pipeline);
            mesh_pass.set_bind_group(0, &frame_buffer.mesh_bind_group, &[]);
            mesh_pass.dispatch_workgroups(u32::try_from(world.chunk_count).unwrap(), 1, 1);
        }
        {
            let mut render_pass = utils::create_render_pass(
                &mut encoder,
                &current_surface_texture_view,
                &self.depth_texture_view,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &frame_buffer.render_bind_group, &[]);
            render_pass.draw_indirect(&frame_buffer.draw_indirect_buffer, 0);
        }

        let submission_index = self.queue.submit(Some(encoder.finish()));
        self.submission_indices[frame_index] = Some(submission_index);
        current_surface_texture.present();
    }
}
