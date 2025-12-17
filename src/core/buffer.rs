use bytemuck::Pod;
use wgpu::util::DeviceExt;

/// A wrapper around a wgpu::Buffer
pub struct VertexBuffer<'a, T: Pod> {
    /// The wrapped buffer
    pub buffer: wgpu::Buffer,
    /// The content of the buffer
    pub content: &'a [T],
    /// The buffer layout
    pub layout: wgpu::VertexBufferLayout<'static>,
}

impl<'a, T: Pod> VertexBuffer<'a, T> {
    /// Create a new vertex buffer with some content
    pub fn new(
        device: &wgpu::Device,
        content: &'a [T],
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> VertexBuffer<'a, T> {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(content),
            usage: wgpu::BufferUsages::VERTEX,
        });

        VertexBuffer {
            buffer,
            content,
            layout,
        }
    }
}

/// Extension trait to create a new VertexBuffer directly from the wgpu::Device
pub trait BufferExt<'a, T: Pod> {
    /// Create a new VertexBuffer initialized with some content
    fn create_vertex_buffer(
        &self,
        content: &'a [T],
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> VertexBuffer<'a, T>;
}

impl<'a, T: Pod> BufferExt<'a, T> for wgpu::Device {
    fn create_vertex_buffer(
        &self,
        content: &'a [T],
        layout: wgpu::VertexBufferLayout<'static>,
    ) -> VertexBuffer<'a, T> {
        VertexBuffer::new(self, content, layout)
    }
}
