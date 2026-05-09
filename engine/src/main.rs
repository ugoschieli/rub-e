use crate::cube::{Cube, CubeGpu};
use crate::frame_buffer::FrameBuffers;
use crate::time::Time;
use camera::Camera;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};
use std::ops::Range;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

mod camera;
mod cube;
mod frame_buffer;
mod time;
mod utils;

const FRAMES_IN_FLIGHT: usize = 2;
const CUBE_NUMBER: usize = 1_000_000;
const CUBE_RANGE: Range<f32> = -1000.0..1000.0;

#[derive(Debug)]
struct Gfx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture_view: wgpu::TextureView,
    frame_buffers: [FrameBuffers; FRAMES_IN_FLIGHT],
    submission_indices: [Option<wgpu::SubmissionIndex>; FRAMES_IN_FLIGHT],
}

impl Gfx {
    fn new(event_loop: &ActiveEventLoop, window: Arc<Window>, cubes: &[Cube]) -> Self {
        let size = window.inner_size();

        let instance = utils::create_instance(event_loop);
        let adapter = utils::create_adapter(&instance);
        let (device, queue) = utils::create_device(&adapter);
        let (surface, surface_config) =
            utils::create_surface(&instance, &adapter, &device, window, size);

        let cubes = cubes
            .iter()
            .map(|cube| cube.to_gpu())
            .collect::<Vec<CubeGpu>>();

        let cubes_buffer = utils::create_buffer_init(
            &device,
            "ETIB Cubes Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            &cubes,
        );

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
            .map(|_| FrameBuffers::new(&device, &cubes, &cubes_buffer, &face_matrices_buffer))
            .collect::<Vec<FrameBuffers>>();

        let compute_pipeline =
            utils::create_compute_pipeline(&device, &frame_buffers[0].compute_bind_group_layout);

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
            compute_pipeline,
            render_pipeline,
            depth_texture_view,
            frame_buffers: frame_buffers.try_into().unwrap(),
            submission_indices: [const { None }; FRAMES_IN_FLIGHT],
        }
    }

    fn render(&mut self, camera: &Camera, size: PhysicalSize<u32>, time: &Time, cubes: &[Cube]) {
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
        {
            let mut compute_pass = utils::create_compute_pass(&mut encoder);
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &frame_buffer.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                u32::try_from(cubes.len().div_ceil(64)).unwrap(),
                1,
                1,
            );
        }
        {
            let mut render_pass = utils::create_render_pass(
                &mut encoder,
                &current_surface_texture_view,
                &self.depth_texture_view,
            );

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &frame_buffer.render_bind_group, &[]);
            #[allow(clippy::cast_possible_truncation)]
            render_pass.draw(0..(cubes.len() as u32 * 36), 0..1);
        }

        let submission_index = self.queue.submit(Some(encoder.finish()));
        self.submission_indices[frame_index] = Some(submission_index);
        current_surface_texture.present();
    }
}

#[derive(Debug)]
struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    camera: Option<Camera>,
    keys_held: HashSet<KeyCode>,
    time: Time,
    cubes: Vec<Cube>,
}

impl App {
    pub fn new() -> Self {
        let cubes = (0..CUBE_NUMBER)
            .map(|_| Cube::random_cube(CUBE_RANGE))
            .collect();

        Self {
            window: None,
            gfx: None,
            camera: None,
            keys_held: HashSet::default(),
            time: Time::new(),
            cubes,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("ETIB")
                        .with_fullscreen(Some(Fullscreen::Borderless(None))),
                )
                .expect("Failed to create a window"),
        );

        window.set_cursor_grab(CursorGrabMode::Locked).unwrap();
        window.set_cursor_visible(false);

        self.gfx = Some(Gfx::new(event_loop, window.clone(), &self.cubes));
        self.window = Some(window);
        self.camera = Some(Camera::new());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys_held.insert(code);
                    } else {
                        self.keys_held.remove(&code);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                const SPEED: f32 = 3.0;

                self.time.tick();

                let camera = self.camera.as_mut().unwrap();
                let forward = camera.rotation * Vec3::NEG_Z;
                let right = camera.rotation * Vec3::X;

                let mut move_dir = Vec3::ZERO;
                if self.keys_held.contains(&KeyCode::KeyW) {
                    move_dir += forward;
                }
                if self.keys_held.contains(&KeyCode::KeyS) {
                    move_dir -= forward;
                }
                if self.keys_held.contains(&KeyCode::KeyD) {
                    move_dir += right;
                }
                if self.keys_held.contains(&KeyCode::KeyA) {
                    move_dir -= right;
                }
                if self.keys_held.contains(&KeyCode::Space) {
                    move_dir += Vec3::Y;
                }
                if self.keys_held.contains(&KeyCode::ShiftLeft) {
                    move_dir -= Vec3::Y;
                }
                camera.position += move_dir.normalize_or_zero() * SPEED * self.time.dt;

                let gfx = self.gfx.as_mut().unwrap();
                let window = self.window.as_ref().unwrap();

                gfx.render(camera, window.inner_size(), &self.time, &self.cubes);

                window.request_redraw();
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        const SENSITIVITY: f32 = 0.05;

        let camera = self.camera.as_mut().unwrap();

        let right = camera.rotation * Vec3::X;

        if let DeviceEvent::MouseMotion { delta } = event {
            #[allow(clippy::cast_possible_truncation)]
            let delta = (delta.0 as f32, delta.1 as f32);
            let yaw = Quat::from_rotation_y(-delta.0.to_radians() * SENSITIVITY);
            let pitch = Quat::from_axis_angle(right, -delta.1.to_radians() * SENSITIVITY);

            let candidate = (pitch * camera.rotation).normalize();
            let new_forward = candidate * Vec3::NEG_Z;
            if new_forward.y.abs() < 0.99 {
                camera.rotation = candidate;
            }
            camera.rotation = (yaw * camera.rotation).normalize();
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
