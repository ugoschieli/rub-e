use crate::constants::{FRAMES_IN_FLIGHT, SENSITIVITY, SPEED};
use crate::gfx::Gfx;
use crate::time::Time;
use crate::utils;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashSet;
use std::f32::consts::FRAC_PI_4;
use winit::dpi::PhysicalSize;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fovy: f32,
    pub buffers: Vec<wgpu::Buffer>,
}

impl Camera {
    pub fn new(gfx: &Gfx) -> Self {
        let mut buffers = Vec::with_capacity(FRAMES_IN_FLIGHT);

        for i in 0..FRAMES_IN_FLIGHT {
            let buffer = utils::create_buffer_init(
                &gfx.device,
                format!("ETIB Camera Buffer {i}").as_str(),
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                &[Mat4::ZERO],
            );

            buffers.push(buffer);
        }

        Self {
            position: Vec3::new(0., 0., 2.),
            rotation: Quat::default(),
            fovy: FRAC_PI_4,
            buffers,
        }
    }

    pub fn upload(&self, gfx: &Gfx, size: PhysicalSize<u32>) {
        let buffer = &self.buffers[gfx.frame_index];

        gfx.queue
            .write_buffer(buffer, 0, bytemuck::bytes_of(&self.matrix(size)));
    }

    pub fn matrix(&self, size: PhysicalSize<u32>) -> Mat4 {
        let forward = self.rotation * -Vec3::Z;

        let view = Mat4::look_at_rh(
            self.position,           // Move the camera back to see the square
            self.position + forward, // Look at the center
            Vec3::Y,                 // Up is Y
        );

        #[allow(clippy::cast_precision_loss)]
        let projection =
            Mat4::perspective_infinite_rh(self.fovy, size.width as f32 / size.height as f32, 0.1);

        let base_change = Mat4::from_cols(
            glam::Vec4::new(1.0, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, -1.0, 0.0),
            glam::Vec4::new(0.0, 1.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        projection * view * base_change
    }

    pub fn handle_keyboard(&mut self, keys_held: &HashSet<KeyCode>, time: &Time) {
        let forward = self.rotation * Vec3::NEG_Z;
        let right = self.rotation * Vec3::X;

        let mut move_dir = Vec3::ZERO;
        if keys_held.contains(&KeyCode::KeyW) {
            move_dir += forward;
        }
        if keys_held.contains(&KeyCode::KeyS) {
            move_dir -= forward;
        }
        if keys_held.contains(&KeyCode::KeyD) {
            move_dir += right;
        }
        if keys_held.contains(&KeyCode::KeyA) {
            move_dir -= right;
        }
        if keys_held.contains(&KeyCode::Space) {
            move_dir += Vec3::Y;
        }
        if keys_held.contains(&KeyCode::ShiftLeft) {
            move_dir -= Vec3::Y;
        }
        self.position += move_dir.normalize_or_zero() * SPEED * time.dt;
    }

    pub fn handle_mouse(&mut self, delta: (f64, f64)) {
        #[allow(clippy::cast_possible_truncation)]
        let delta = (delta.0 as f32, delta.1 as f32);
        let right = self.rotation * Vec3::X;
        let yaw = Quat::from_rotation_y(-delta.0.to_radians() * SENSITIVITY);
        let pitch = Quat::from_axis_angle(right, -delta.1.to_radians() * SENSITIVITY);

        let candidate = (pitch * self.rotation).normalize();
        let new_forward = candidate * Vec3::NEG_Z;
        if new_forward.y.abs() < 0.99 {
            self.rotation = candidate;
        }
        self.rotation = (yaw * self.rotation).normalize();
    }
}
