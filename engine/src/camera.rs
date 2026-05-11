use glam::{Mat4, Quat, Vec3};
use std::f32::consts::FRAC_PI_4;
use winit::dpi::PhysicalSize;

#[derive(Debug, Copy, Clone, Default)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fovy: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0., 0., 2.),
            rotation: Quat::default(),
            fovy: FRAC_PI_4,
        }
    }

    pub fn matrix(&self, size: PhysicalSize<u32>) -> Mat4 {
        let forward = self.rotation * Vec3::new(0., 0., -1.);

        let view = Mat4::look_at_rh(
            self.position,           // Move the camera back to see the square
            self.position + forward, // Look at the center
            Vec3::Y,                 // Up is Y
        );

        #[allow(clippy::cast_precision_loss)]
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
