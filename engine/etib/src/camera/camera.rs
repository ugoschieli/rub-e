use cgmath::{InnerSpace, Matrix, SquareMatrix};
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

use etib_core::bindgroup::{BindGroup, BindGroupBuilder};

/// Enum representing the different camera modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// First-person camera mode
    FirstPerson,
    /// Isometric camera mode
    Isometric,
}

/// Enum representing the different projection types
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Perspective projection
    Perspective {
        /// Field of view in the Y direction (vertical angle)
        fovy: f32,
    },
    /// Orthographic projection
    Orthographic {
        /// Scale factor for the projection
        scale: f32,
    },
}

/// The camera struct
pub struct Camera {
    /// The position of the camera
    pub eye: cgmath::Point3<f32>,
    /// The direction where the camera looks
    pub target: cgmath::Point3<f32>,
    /// The vector of the up direction
    pub up: cgmath::Vector3<f32>,
    /// The aspect ratio (16:9, 4:3, ...) can be calculated with window_width / window_size
    pub aspect: f32,
    /// The camera projection type
    pub projection: Projection,
    /// The near clipping plane
    pub znear: f32,
    /// The far clipping plane
    pub zfar: f32,
    /// The camera matrix (projection * view)
    pub matrix: cgmath::Matrix4<f32>,
    /// The previous camera matrix (for occlusion culling reprojection)
    pub prev_matrix: cgmath::Matrix4<f32>,
    /// The bind group associated with the matrix
    pub bind_group: BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    pub view_proj: [[f32; 4]; 4],
    pub inv_view: [[f32; 4]; 4],
    pub inv_proj: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
}

/// Camera controller for FPS-style keyboard and mouse input
///
/// # Example
/// ```no_run
/// use etib::camera::{CameraController, CameraMode};
///
/// // Create a controller with speed=10.0 units/sec and sensitivity=0.003
/// let mut controller = CameraController::new(10.0, 0.003);
/// controller.mode = CameraMode::Isometric; // or FirstPerson
///
/// // In your event loop:
/// // - Call process_keyboard() for KeyboardInput events
/// // - Call process_mouse() for MouseMotion events  
/// // - Call process_scroll() for MouseWheel events
/// // - Call update_camera() in your render function
///
/// // Controls:
/// // - WASD or Arrow keys: Move forward/back/left/right (pans in Isometric mode)
/// // - Space: Move up
/// // - Shift: Move down
/// // - Mouse: Look around (FPS-style view control, disabled in Isometric)
/// // - Scroll: Adjust movement speed (FPS) or zoom (Isometric)
/// ```
pub struct CameraController {
    /// Movement speed in units per second
    pub speed: f32,
    /// Mouse sensitivity
    pub sensitivity: f32,
    /// Current yaw angle (rotation around Y axis) in radians
    pub yaw: f32,
    /// Current pitch angle (rotation around X axis) in radians
    pub pitch: f32,
    /// Forward movement amount (-1.0 to 1.0)
    forward_amount: f32,
    /// Right movement amount (-1.0 to 1.0)
    right_amount: f32,
    /// Up movement amount (-1.0 to 1.0)
    up_amount: f32,
    /// Camera mode (FirstPerson or Isometric)
    pub mode: CameraMode,
    /// Zoom factor for orthographic camera
    zoom: f32,
}

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    /// Create a new camera object
    pub fn new(
        device: &wgpu::Device,
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        projection: Projection,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = cgmath::Matrix4::look_at_rh(eye, target, up);
        let proj = match projection {
            Projection::Perspective { fovy } => {
                cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar)
            }
            Projection::Orthographic { scale } => {
                let right = scale * aspect;
                let left = -right;
                let top = scale;
                let bottom = -top;
                cgmath::ortho(left, right, bottom, top, znear, zfar)
            }
        };
        let matrix = OPENGL_TO_WGPU_MATRIX * proj * view;

        let camera_raw = CameraRaw {
            view_proj: matrix.into(),
            inv_view: view.transpose().into(), // The view matrix is orthonormal its invert is equal to the transpose
            inv_proj: proj.invert().unwrap().into(),
            prev_view_proj: matrix.into(), // Initialize prev with current
        };

        let bind_group = BindGroupBuilder::new()
            .add_uniform_buffer(
                device,
                0,
                bytemuck::bytes_of(&camera_raw),
                wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
            )
            .build(device, Some("Camera Bind Group"));

        Self {
            eye,
            target,
            up,
            aspect,
            projection,
            znear,
            zfar,
            matrix,
            prev_matrix: matrix,
            bind_group,
        }
    }

    /// Update the camera matrix with the new camera position
    pub fn update_matrix(&mut self, queue: &wgpu::Queue) -> cgmath::Matrix4<f32> {
        // Store current matrix as previous before updating
        self.prev_matrix = self.matrix;

        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = match self.projection {
            Projection::Perspective { fovy } => {
                cgmath::perspective(cgmath::Deg(fovy), self.aspect, self.znear, self.zfar)
            }
            Projection::Orthographic { scale } => {
                let right = self.aspect * scale;
                let left = -right;
                let top = scale;
                let bottom = -top;
                cgmath::ortho(left, right, bottom, top, self.znear, self.zfar)
            }
        };

        let view_proj = OPENGL_TO_WGPU_MATRIX * proj * view;
        self.matrix = view_proj;

        let camera_raw = CameraRaw {
            view_proj: view_proj.into(),
            inv_proj: proj.invert().unwrap().into(),
            inv_view: view.transpose().into(), // The view matrix is orthonormal its invert is equal to the transpose
            prev_view_proj: self.prev_matrix.into(),
        };
        self.bind_group
            .write_buffer(&queue, 0, bytemuck::bytes_of(&camera_raw));

        view_proj
    }
}

impl CameraController {
    /// Create a new camera controller
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            yaw: -std::f32::consts::FRAC_PI_2, // Start looking forward (-Z direction)
            pitch: 0.0,
            forward_amount: 0.0,
            right_amount: 0.0,
            up_amount: 0.0,
            mode: CameraMode::FirstPerson,
            zoom: 1.0,
        }
    }

    /// Process keyboard input events
    pub fn process_keyboard(&mut self, key: KeyEvent) -> bool {
        let amount = if key.state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key.physical_key {
            PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.forward_amount = amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.forward_amount = -amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.right_amount = -amount;
                true
            }
            PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.right_amount = amount;
                true
            }
            PhysicalKey::Code(KeyCode::Space) => {
                self.up_amount = amount;
                true
            }
            PhysicalKey::Code(KeyCode::ShiftLeft) | PhysicalKey::Code(KeyCode::ShiftRight) => {
                self.up_amount = -amount;
                true
            }
            _ => false,
        }
    }

    /// Process mouse movement for camera rotation
    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        if self.mode == CameraMode::FirstPerson {
            self.yaw += delta_x as f32 * self.sensitivity;
            self.pitch -= delta_y as f32 * self.sensitivity;

            // Clamp pitch to avoid gimbal lock
            self.pitch = self.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.1,
                std::f32::consts::FRAC_PI_2 - 0.1,
            );
        }
    }

    /// Process mouse scroll for speed adjustment
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        let change = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
        };

        match self.mode {
            CameraMode::FirstPerson => {
                self.speed = (self.speed + change * 0.5).max(0.1);
            }
            CameraMode::Isometric => {
                if change > 0.0 {
                    self.zoom = 1.0 / 1.1; // Zoom in
                } else if change < 0.0 {
                    self.zoom = 1.1; // Zoom out
                }
            }
        }
    }

    /// Update camera position and target based on controller state
    pub fn update_camera(&mut self, queue: &wgpu::Queue, camera: &mut Camera, dt: f32) {
        match self.mode {
            CameraMode::FirstPerson => {
                // Calculate forward direction from yaw and pitch (FPS-style looking)
                let forward = cgmath::Vector3::new(
                    self.yaw.cos() * self.pitch.cos(),
                    self.pitch.sin(),
                    self.yaw.sin() * self.pitch.cos(),
                )
                .normalize();

                // Calculate right direction (perpendicular to forward and world up)
                let world_up = cgmath::Vector3::unit_y();
                let right = forward.cross(world_up).normalize();

                // Calculate movement forward direction (parallel to ground for FPS movement)
                let forward_movement = cgmath::Vector3::new(forward.x, 0.0, forward.z).normalize();

                // Update camera position based on input
                let speed_delta = self.speed * dt;
                camera.eye += forward_movement * self.forward_amount * speed_delta;
                camera.eye += right * self.right_amount * speed_delta;
                camera.eye += world_up * self.up_amount * speed_delta;

                // Update camera target to be one unit ahead in the looking direction
                camera.target = camera.eye + forward;

                // Keep up vector as world up for FPS-style camera
                camera.up = world_up;
            }
            CameraMode::Isometric => {
                let speed_delta = self.speed * dt;
                let pan_x = self.right_amount * speed_delta;
                let pan_z = -self.forward_amount * speed_delta; // W moves "north" (-Z)

                camera.eye.x += pan_x;
                camera.eye.z += pan_z;
                camera.target.x += pan_x;
                camera.target.z += pan_z;

                if let Projection::Orthographic { ref mut scale } = camera.projection {
                    *scale *= self.zoom;
                    self.zoom = 1.0; // Reset zoom factor
                }
            }
        }

        camera.update_matrix(queue);
    }
}
