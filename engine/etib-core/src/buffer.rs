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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        if let Ok(adapter) = adapter {
            Some(
                pollster::block_on(
                    adapter.request_device(&wgpu::DeviceDescriptor::default()),
                )
                .unwrap(),
            )
        } else {
            None
        }
    }

    fn empty_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[],
        }
    }

    #[test]
    fn test_vertex_buffer_new_stores_content() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let vertices = [1.0f32, 2.0, 3.0, 4.0];
        let vb = VertexBuffer::new(&device, &vertices, empty_layout());
        assert_eq!(vb.content, &vertices);
    }

    #[test]
    fn test_vertex_buffer_buffer_size() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let vertices = [1.0f32, 2.0, 3.0, 4.0];
        let vb = VertexBuffer::new(&device, &vertices, empty_layout());
        let expected = (vertices.len() * std::mem::size_of::<f32>()) as u64;
        assert_eq!(vb.buffer.size(), expected);
    }

    #[test]
    fn test_vertex_buffer_stores_layout() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let vertices = [0.0f32];
        let layout = empty_layout();
        let vb = VertexBuffer::new(&device, &vertices, layout.clone());
        assert_eq!(vb.layout.array_stride, layout.array_stride);
        assert_eq!(vb.layout.step_mode, layout.step_mode);
    }

    #[test]
    fn test_buffer_ext_create_vertex_buffer() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let vertices = [0.0f32, 1.0, 2.0];
        let vb = device.create_vertex_buffer(&vertices, empty_layout());
        assert_eq!(vb.content, &vertices);
        assert_eq!(
            vb.buffer.size(),
            (vertices.len() * std::mem::size_of::<f32>()) as u64
        );
    }
}
