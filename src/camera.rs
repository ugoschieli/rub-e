use crate::uniform::Uniform;
use wgpu::util::DeviceExt;

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
