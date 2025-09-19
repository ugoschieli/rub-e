use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::Shader;
use crate::buffer::BufferExt;
use crate::camera::Camera;
use crate::pipeline::Pipeline;
use crate::vertex;

/// A wrapper around muliple wgpu structs that holds the graphics state of the app
pub struct Gfx {
    /// The wgpu Instance
    pub instance: wgpu::Instance,
    /// The wgpu Surface
    pub surface: wgpu::Surface<'static>,
    /// The wgpu SurfaceConfiguration
    pub surface_config: wgpu::SurfaceConfiguration,
    /// The wgpu device
    pub device: wgpu::Device,
    /// The wgpu Queue
    pub queue: wgpu::Queue,
    /// The wgpu RenderPipeline
    pub pipeline: Pipeline,
    /// A vertex Buffer
    pub vertex_buffer: wgpu::Buffer,
    /// An index Buffer
    pub index_buffer: wgpu::Buffer,
    /// The camera
    pub camera: Camera,
    /// The Depth Buffer Texture View
    pub depth_texture_view: wgpu::TextureView,
}

impl Gfx {
    /// Create a new wgpu Instance and initialize the Gfx struct.
    /// Need to be called only once at the initialization of an app.
    pub fn new(window: Arc<Window>) -> Gfx {
        let window_size = window.inner_size();

        let instance_desc = wgpu::InstanceDescriptor::default();
        let instance = wgpu::Instance::new(&instance_desc);
        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let surface_config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let camera = Camera::new(
            &device,
            (2.0, 2.0, 3.0).into(),
            (0.0, 0.0, 0.0).into(),
            cgmath::Vector3::unit_y(),
            window_size.width as f32 / window_size.height as f32,
            45.0,
            0.1,
            100.0,
        );

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let vertex_buffer = device.create_vertex_buffer(vertex::VERTICES, vertex::Vertex::desc());
        // let vertex_buffer = device.create_vertex_buffer(vertex::VERTICES2, vertex::Vertex::desc());

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(vertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let shader_str = include_str!("./shaders/shader.wgsl");
        let shader = Shader::new(shader_str, &device, None);

        let pipeline = Pipeline::new(
            &device,
            &[&camera.uniform.layout],
            &shader,
            &surface_config,
            &vertex_buffer,
        );

        Gfx {
            instance,
            surface,
            surface_config,
            device,
            queue,
            pipeline,
            vertex_buffer: vertex_buffer.buffer,
            index_buffer,
            camera,
            depth_texture_view,
        }
    }

    /// Get the texture and view of the next frame that will be rendered
    pub fn get_next_frame(&self) -> (wgpu::SurfaceTexture, wgpu::TextureView) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        (frame, view)
    }

    /// Get a default wgpu::RenderPassDescriptor with depth testing enabled
    pub fn render_pass<'gfx: 'tex, 'tex>(
        &'gfx self,
        color_attachments: &'tex [Option<wgpu::RenderPassColorAttachment<'tex>>],
    ) -> wgpu::RenderPassDescriptor<'tex> {
        wgpu::RenderPassDescriptor {
            label: None,
            color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        }
    }

    /// Helper function to create a default wgpu::RenderPassDescriptor from a wgpu::TextureView
    pub fn color_attachments_from_view<'tex>(
        view: &'tex wgpu::TextureView,
    ) -> wgpu::RenderPassColorAttachment<'tex> {
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        }
    }

    /// Helper function to reconfigure the Surface size. Needs to be called when the window is
    /// resized.
    pub fn reconfigure_surface_size(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }
}
