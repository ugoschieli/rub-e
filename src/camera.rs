use crate::uniform::Uniform;
use cgmath::InnerSpace;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, KeyEvent, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};

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
    /// The camera field of view
    pub fovy: f32,
    /// The near clipping plane
    pub znear: f32,
    /// The far clipping plane
    pub zfar: f32,
    /// The camera matrix (projection * view)
    pub matrix: cgmath::Matrix4<f32>,
    /// The buffer storing the matrix
    pub buffer: wgpu::Buffer,
    /// The uniform associated with the matrix
    pub uniform: Uniform,
}

/// Camera controller for FPS-style keyboard and mouse input
/// 
/// # Example
/// ```no_run
/// use etib::CameraController;
/// 
/// // Create a controller with speed=10.0 units/sec and sensitivity=0.003
/// let mut controller = CameraController::new(10.0, 0.003);
/// 
/// // In your event loop:
/// // - Call process_keyboard() for KeyboardInput events
/// // - Call process_mouse() for MouseMotion events  
/// // - Call process_scroll() for MouseWheel events
/// // - Call update_camera() in your render function
/// 
/// // Controls:
/// // - WASD or Arrow keys: Move forward/back/left/right
/// // - Space: Move up
/// // - Shift: Move down
/// // - Mouse: Look around (FPS-style view control)
/// // - Scroll: Adjust movement speed
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
}

#[rustfmt::skip]
/// The cgmath crate use the OpenGL matrix format multiplying the camera matrix by this one convert
/// it to the WebGPU matrix format (same as DX12 and Vulkan)
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    /// Create a new camera object
    pub fn new(
        device: &wgpu::Device,
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let view = cgmath::Matrix4::look_at_rh(eye, target, up);
        let proj = cgmath::perspective(cgmath::Deg(fovy), aspect, znear, zfar);
        let matrix = OPENGL_TO_WGPU_MATRIX * proj * view;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[Into::<[[f32; 4]; 4]>::into(matrix)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform = Uniform::new(
            device,
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            },
            vec![buffer.as_entire_binding()],
            None,
        );

        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            matrix,
            buffer,
            uniform,
        }
    }

    /// Update the camera matrix with the new camera position
    pub fn update_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
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
        self.yaw += delta_x as f32 * self.sensitivity;
        self.pitch -= delta_y as f32 * self.sensitivity;

        // Clamp pitch to avoid gimbal lock
        self.pitch = self.pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
    }

    /// Process mouse scroll for speed adjustment
    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        let change = match delta {
            MouseScrollDelta::LineDelta(_, y) => *y * 0.5,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
        };
        self.speed = (self.speed + change).max(0.1);
    }

    /// Update camera position and target based on controller state (FPS-style)
    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
        // Calculate forward direction from yaw and pitch (FPS-style looking)
        let forward = cgmath::Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();

        // Calculate right direction (perpendicular to forward and world up)
        // Use world up (Y axis) for FPS-style movement
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
}
