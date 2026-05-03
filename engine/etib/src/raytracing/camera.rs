use super::{InputState, renderer::Renderer};

/// First-person camera for the raytracing pipeline.
pub struct Camera {
    /// World-space eye position.
    pub pos: glam::Vec3,
    /// Horizontal rotation angle in radians.
    pub yaw: f32,
    /// Vertical rotation angle in radians.
    pub pitch: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// GPU-compatible camera uniform block uploaded to the compute shader each frame.
pub struct CameraUniforms {
    /// Ray origin (camera position).
    pub center: glam::Vec3,
    _pad0: f32,
    /// World-space position of the top-left pixel center.
    pub pixel00_loc: glam::Vec3,
    _pad1: f32,
    /// Horizontal step vector per pixel in world space.
    pub pixel_delta_u: glam::Vec3,
    _pad2: f32,
    /// Vertical step vector per pixel in world space.
    pub pixel_delta_v: glam::Vec3,
    _pad3: f32,
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Inverse of the view-projection matrix.
    pub inv_view_proj: [[f32; 4]; 4],
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: glam::Vec3::new(0.0, 0.0, 0.0),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
        }
    }
}

impl Camera {
    /// Move the camera based on the current input state. Resets `frame_count` on movement.
    pub fn update(&mut self, input: &InputState, speed: f32, frame_count: &mut u32) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();

        let forward =
            glam::Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        let right = forward.cross(glam::Vec3::Y).normalize();

        let old_pos = self.pos;

        if input.forward {
            self.pos += forward * speed;
        }
        if input.backward {
            self.pos -= forward * speed;
        }
        if input.left {
            self.pos -= right * speed;
        }
        if input.right {
            self.pos += right * speed;
        }
        if input.up {
            self.pos += glam::Vec3::Y * speed;
        }
        if input.down {
            self.pos -= glam::Vec3::Y * speed;
        }

        if old_pos != self.pos {
            *frame_count = 0;
        }
    }

    /// Compute and return the GPU uniform block for the current camera state.
    pub fn update_uniforms(&self, renderer: &Renderer) -> CameraUniforms {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let forward =
            glam::Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        let right = forward.cross(glam::Vec3::Y).normalize();
        let up = right.cross(forward).normalize();
        let width = renderer.surface_config().width as f32;
        let height = renderer.surface_config().height as f32;
        let aspect_ratio = width / height;

        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * aspect_ratio;
        let viewport_u = right * viewport_width;
        let viewport_v = -up * viewport_height;

        let pixel_delta_u = viewport_u / width;
        let pixel_delta_v = viewport_v / height;
        let viewport_upper_left =
            self.pos + (forward * focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let view = glam::Mat4::look_to_rh(self.pos, forward, glam::Vec3::Y);
        let proj =
            glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 1000.0);
        let view_proj = (proj * view).to_cols_array_2d();
        let inv_view_proj = (proj * view).inverse().to_cols_array_2d();

        CameraUniforms {
            center: self.pos,
            _pad0: 0.0,
            pixel00_loc,
            _pad1: 0.0,
            pixel_delta_u,
            _pad2: 0.0,
            pixel_delta_v,
            _pad3: 0.0,
            view_proj,
            inv_view_proj,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let cam = Camera::default();
        assert_eq!(cam.pos, glam::Vec3::ZERO);
        assert_eq!(cam.yaw, -std::f32::consts::FRAC_PI_2);
        assert_eq!(cam.pitch, 0.0);
    }

    #[test]
    fn test_camera_update() {
        let mut cam = Camera::default();
        let mut frame_count = 10;
        let speed = 1.0;

        let mut input = InputState::default();
        input.forward = true;

        cam.update(&input, speed, &mut frame_count);
        
        // Since yaw is -PI/2 (facing -Z), forward should be -Z
        assert!(cam.pos.z < 0.0);
        assert_eq!(frame_count, 0); // Frame count is reset when moving
        
        // Move back
        let old_pos = cam.pos;
        input.forward = false;
        input.backward = true;
        
        frame_count = 10;
        cam.update(&input, speed, &mut frame_count);
        assert!(cam.pos.z > old_pos.z);
        assert_eq!(frame_count, 0);

        // No input, frame count should not reset
        input.backward = false;
        frame_count = 10;
        cam.update(&input, speed, &mut frame_count);
        assert_eq!(frame_count, 10);
    }
}
