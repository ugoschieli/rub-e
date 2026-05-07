use std::collections::HashMap;

use wgpu::util::DeviceExt;

/// Wrapper around a wgpu::BindGroup to represent a Uniform value
#[derive(Debug, Clone)]
pub struct BindGroup {
    /// The layout of the Uniform
    pub layout: wgpu::BindGroupLayout,
    /// The wrapped wgpu::BindGroup
    pub bind_group: wgpu::BindGroup,
    /// The underlying data
    pub resources: HashMap<u32, WrappedResource>,
}

/// Enum representing the different types of resources that can be bound to a bind group
#[derive(Debug, Clone)]
pub enum WrappedResource {
    /// A buffer
    Buffer(wgpu::Buffer),
    /// A texture view
    TextureView(wgpu::TextureView),
    /// A texture sampler
    Sampler(wgpu::Sampler),
}

impl BindGroup {
    /// Write data to a buffer in the bind group
    pub fn write_buffer(&self, queue: &wgpu::Queue, binding: u32, data: &[u8]) {
        if let Some(WrappedResource::Buffer(buffer)) = self.resources.get(&binding) {
            queue.write_buffer(buffer, 0, data);
        } else {
            eprintln!("Warning: No buffer found at binding {}", binding);
        }
    }

    /// Get a resource from the bind group
    pub fn data(&self, binding: u32) -> Option<&WrappedResource> {
        self.resources.get(&binding)
    }

    /// Get a buffer from the bind group
    pub fn get_buffer(&self, binding: u32) -> Option<&wgpu::Buffer> {
        match self.data(binding)? {
            WrappedResource::Buffer(buffer) => Some(buffer),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
/// Fluent builder for BindGroup
pub struct BindGroupBuilder {
    resources: HashMap<u32, WrappedResource>,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupBuilder {
    /// Create a new fluent BindGroupBuilder
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            layout_entries: Vec::new(),
        }
    }

    /// Create and add a uniform buffer containing the data to the bind group
    pub fn add_uniform_buffer(
        mut self,
        device: &wgpu::Device,
        binding: u32,
        data: &[u8],
        visibility: wgpu::ShaderStages,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Buffer Binding {}", binding)),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.resources
            .insert(binding, WrappedResource::Buffer(buffer));

        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    /// Add a texture view to the bind group
    pub fn add_texture(
        mut self,
        binding: u32,
        view: wgpu::TextureView,
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
    ) -> Self {
        // Note: We take ownership of the view here
        self.resources
            .insert(binding, WrappedResource::TextureView(view));

        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type,
            },
            count: None,
        });
        self
    }

    /// Add a cube texture view to the bind group
    pub fn add_cube_texture(
        mut self,
        binding: u32,
        view: wgpu::TextureView,
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
    ) -> Self {
        // Note: We take ownership of the view here
        self.resources
            .insert(binding, WrappedResource::TextureView(view));

        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::Cube,
                sample_type,
            },
            count: None,
        });
        self
    }

    /// Add a sampler to the bind group
    pub fn add_sampler(
        mut self,
        binding: u32,
        sampler: wgpu::Sampler,
        filtering: wgpu::SamplerBindingType,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        self.resources
            .insert(binding, WrappedResource::Sampler(sampler));

        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(filtering),
            count: None,
        });
        self
    }

    /// Add a storage texture to the bind group
    pub fn add_storage_texture(
        mut self,
        binding: u32,
        view: wgpu::TextureView,
        visibility: wgpu::ShaderStages,
    ) -> Self {
        // Note: We take ownership of the view here
        self.resources
            .insert(binding, WrappedResource::TextureView(view));

        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        });
        self
    }

    /// Build the bind group. Call this when you are done adding resources.
    pub fn build(self, device: &wgpu::Device, label: Option<&str>) -> BindGroup {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &self.layout_entries,
        });

        let mut entries = Vec::new();

        for (binding, resource) in &self.resources {
            let wgpu_resource = match resource {
                WrappedResource::Buffer(b) => b.as_entire_binding(),
                WrappedResource::TextureView(v) => wgpu::BindingResource::TextureView(v),
                WrappedResource::Sampler(s) => wgpu::BindingResource::Sampler(s),
            };

            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu_resource,
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &layout,
            entries: &entries,
        });

        BindGroup {
            layout,
            bind_group,
            resources: self.resources,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bindgroup_builder_headless() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
            
            let data = [1.0f32, 2.0, 3.0, 4.0];
            let bytes = bytemuck::cast_slice(&data);

            let bg = BindGroupBuilder::new()
                .add_uniform_buffer(&device, 0, bytes, wgpu::ShaderStages::COMPUTE)
                .build(&device, Some("test bg"));
            
            assert!(bg.get_buffer(0).is_some());
            assert!(bg.get_buffer(1).is_none());

            let new_data = [5.0f32, 6.0, 7.0, 8.0];
            bg.write_buffer(&queue, 0, bytemuck::cast_slice(&new_data));
            
            // Warning condition
            bg.write_buffer(&queue, 1, bytemuck::cast_slice(&new_data));
        }
    }

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

    #[test]
    fn test_data_returns_buffer_resource() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let data = [0.0f32; 4];
        let bg = BindGroupBuilder::new()
            .add_uniform_buffer(
                &device,
                0,
                bytemuck::cast_slice(&data),
                wgpu::ShaderStages::VERTEX,
            )
            .build(&device, None);
        assert!(matches!(bg.data(0), Some(WrappedResource::Buffer(_))));
        assert!(bg.data(1).is_none());
    }

    #[test]
    fn test_get_buffer_returns_none_for_non_buffer_binding() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = BindGroupBuilder::new()
            .add_texture(
                0,
                view,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .build(&device, None);
        assert!(bg.get_buffer(0).is_none());
    }

    #[test]
    fn test_add_texture_binding() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = BindGroupBuilder::new()
            .add_texture(
                0,
                view,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .build(&device, None);
        assert!(matches!(bg.data(0), Some(WrappedResource::TextureView(_))));
    }

    #[test]
    fn test_add_sampler_binding() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let bg = BindGroupBuilder::new()
            .add_sampler(
                0,
                sampler,
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(&device, None);
        assert!(matches!(bg.data(0), Some(WrappedResource::Sampler(_))));
    }

    #[test]
    fn test_add_cube_texture_binding() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });
        let bg = BindGroupBuilder::new()
            .add_cube_texture(
                0,
                view,
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .build(&device, None);
        assert!(matches!(bg.data(0), Some(WrappedResource::TextureView(_))));
    }

    #[test]
    fn test_add_storage_texture_binding() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bg = BindGroupBuilder::new()
            .add_storage_texture(0, view, wgpu::ShaderStages::COMPUTE)
            .build(&device, None);
        assert!(matches!(bg.data(0), Some(WrappedResource::TextureView(_))));
    }

    #[test]
    fn test_multiple_uniform_buffers() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let data_a = [1.0f32, 2.0];
        let data_b = [3.0f32, 4.0, 5.0, 6.0];
        let bg = BindGroupBuilder::new()
            .add_uniform_buffer(
                &device,
                0,
                bytemuck::cast_slice(&data_a),
                wgpu::ShaderStages::VERTEX,
            )
            .add_uniform_buffer(
                &device,
                1,
                bytemuck::cast_slice(&data_b),
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(&device, None);
        assert!(bg.get_buffer(0).is_some());
        assert!(bg.get_buffer(1).is_some());
        assert!(bg.get_buffer(2).is_none());
    }
}
