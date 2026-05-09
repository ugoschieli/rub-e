use std::collections::HashSet;
use std::f32::consts::{FRAC_PI_2, PI};
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

use crate::time::Time;
use camera::Camera;

mod camera;
mod time;
mod utils;

const CUBES: &[Cube] = &[
    Cube::new(Vec3::new(0., 0., -5.), Vec3::Y),
    Cube::new(Vec3::new(3., 0., -5.), Vec3::Z),
];

#[derive(Debug, Copy, Clone)]
struct Cube {
    position: Vec3,
    color: Vec3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct CubeGpu {
    position: Vec4,
    color: Vec4,
}

impl Cube {
    pub const fn new(position: Vec3, color: Vec3) -> Self {
        Self { position, color }
    }

    pub fn to_gpu(self) -> CubeGpu {
        CubeGpu {
            position: Vec4::ZERO.with_xyz(self.position),
            color: Vec4::ONE.with_xyz(self.color),
        }
    }
}

#[derive(Debug)]
struct Gfx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    compute_pipeline: wgpu::ComputePipeline,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    compute_bind_group: wgpu::BindGroup,
    bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
}

impl Gfx {
    fn new(event_loop: &ActiveEventLoop, window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = utils::create_instance(event_loop);
        let adapter = utils::create_adapter(&instance);
        let (device, queue) = utils::create_device(&adapter);
        let (surface, surface_config) =
            utils::create_surface(&instance, &adapter, &device, window, size);

        let camera_buffer = utils::create_buffer_init(
            &device,
            "ETIB Camera Buffer",
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[Mat4::ZERO],
        );

        let cubes = CUBES
            .iter()
            .map(|cube| cube.to_gpu())
            .collect::<Vec<CubeGpu>>();

        let cubes_buffer = utils::create_buffer_init(
            &device,
            "ETIB Cubes Buffer",
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            &cubes,
        );

        let faces_buffer = utils::create_buffer(
            &device,
            "EITB Faces Buffer",
            (cubes.len() * 6 * 48) as u64, // 6 Faces max foreach cubes, a Face is 48 bytes (2 vec4<f32> + u32 + padding)
            wgpu::BufferUsages::STORAGE,
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

        let (compute_bind_group, compute_bind_group_layout) =
            utils::create_compute_bind_group(&device, &cubes_buffer, &faces_buffer);

        let (bind_group, bind_group_layout) = utils::create_bind_group(
            &device,
            &camera_buffer,
            &faces_buffer,
            &face_matrices_buffer,
        );

        let compute_pipeline = utils::create_compute_pipeline(&device, &compute_bind_group_layout);
        let pipeline = utils::create_render_pipeline(&device, &surface_config, &bind_group_layout);
        let (_depth_texture, depth_texture_view) = utils::create_depth_texture(&device, size);

        Self {
            device,
            queue,
            surface,
            compute_pipeline,
            pipeline,
            camera_buffer,
            compute_bind_group,
            bind_group,
            depth_texture_view,
        }
    }

    fn render(&self, camera: &Camera, size: PhysicalSize<u32>) {
        let Some((current_surface_texture, current_surface_texture_view)) =
            utils::get_current_surface_texture(&self.surface)
        else {
            return;
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera.matrix(size)),
        );

        let mut encoder = utils::create_encoder(&self.device);
        {
            let mut compute_pass = utils::create_compute_pass(&mut encoder);
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(
                u32::try_from(CUBES.len().div_ceil(64)).unwrap(),
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

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            #[allow(clippy::cast_possible_truncation)]
            render_pass.draw(0..(CUBES.len() as u32 * 36), 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
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
        Self {
            window: None,
            gfx: None,
            camera: None,
            keys_held: HashSet::default(),
            time: Time::new(),
            cubes: CUBES.into(),
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

        self.gfx = Some(Gfx::new(event_loop, window.clone()));
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

                let gfx = self.gfx.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();

                gfx.render(camera, window.inner_size());

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
