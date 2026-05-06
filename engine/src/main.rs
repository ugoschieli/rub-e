use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use std::sync::Arc;

use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

mod utils;

struct Camera {
    position: Vec3,
    rotation: Quat,
    fovy: f32,
}

impl Camera {
    fn new() -> Self {
        Camera {
            position: Vec3::new(0., 0., 2.),
            rotation: Quat::default(),
            fovy: FRAC_PI_4,
        }
    }

    fn matrix(&self, size: PhysicalSize<u32>) -> Mat4 {
        let forward = self.rotation * Vec3::new(0., 0., -1.);

        let view = Mat4::look_at_rh(
            self.position,           // Move the camera back to see the square
            self.position + forward, // Look at the center
            Vec3::Y,                 // Up is Y
        );

        let projection =
            Mat4::perspective_infinite_rh(self.fovy, size.width as f32 / size.height as f32, 0.1);

        // WGPU uses a 0.0 to 1.0 depth range, while glam's projection matrices
        // target the -1.0 to 1.0 range used by OpenGL. We need to remap it.
        let correction = Mat4::from_cols(
            glam::Vec4::new(1.0, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 1.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 0.5, 0.0),
            glam::Vec4::new(0.0, 0.0, 0.5, 1.0),
        );

        correction * projection * view
    }
}

struct Gfx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
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

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ETIB Camera Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::bytes_of(&Mat4::ZERO),
        });

        let (bind_group, bind_group_layout) = utils::create_bind_group(&device, &camera_buffer);
        let pipeline = utils::create_render_pipeline(&device, &surface_config, &bind_group_layout);
        let (_depth_texture, depth_texture_view) = utils::create_depth_texture(&device, size);

        Gfx {
            device,
            queue,
            surface,
            pipeline,
            camera_buffer,
            bind_group,
            depth_texture_view,
        }
    }

    fn render(&self, camera: &Camera, size: PhysicalSize<u32>) {
        let (current_surface_texture, current_surface_texture_view) =
            match utils::get_current_surface_texture(&self.surface) {
                Some(current_surface) => current_surface,
                None => {
                    return;
                }
            };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera.matrix(size)),
        );

        let mut encoder = utils::create_encoder(&self.device);
        {
            let mut render_pass = utils::create_render_pass(
                &mut encoder,
                &current_surface_texture_view,
                &self.depth_texture_view,
            );

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        current_surface_texture.present();
    }
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gfx: Option<Gfx>,
    camera: Option<Camera>,
    keys_held: HashSet<KeyCode>,
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let camera = self.camera.as_mut().unwrap();

        const SENSITIVITY: f32 = 0.1;

        let right = camera.rotation * Vec3::X;

        if let DeviceEvent::MouseMotion { delta } = event {
            let yaw = Quat::from_rotation_y(-delta.0.to_radians() as f32 * SENSITIVITY);
            let pitch = Quat::from_axis_angle(right, -delta.1.to_radians() as f32 * SENSITIVITY);

            let candidate = (pitch * camera.rotation).normalize();
            let new_forward = candidate * Vec3::NEG_Z;
            if new_forward.y.abs() < 0.99 {
                camera.rotation = candidate;
            }
            camera.rotation = (yaw * camera.rotation).normalize();
        }
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
                const SPEED: f32 = 0.05;

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
                camera.position += move_dir.normalize_or_zero() * SPEED;

                let gfx = self.gfx.as_ref().unwrap();
                let window = self.window.as_ref().unwrap();

                gfx.render(camera, window.inner_size());

                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
