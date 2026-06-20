use crate::constants::FRAMES_IN_FLIGHT;

/// Use when double buffering isn't necessary
#[derive(Debug)]
pub struct BindGroupBuilder<'a> {
    device: &'a wgpu::Device,
    entries: Vec<(u32, wgpu::BindingType, wgpu::BindingResource<'a>)>,
    visibility: Option<wgpu::ShaderStages>,
    label: Option<&'a str>,
}

impl<'a> BindGroupBuilder<'a> {
    pub const fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            entries: vec![],
            visibility: Some(wgpu::ShaderStages::all()),
            label: None,
        }
    }

    #[must_use]
    pub const fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = Some(visibility);
        self
    }

    #[must_use]
    pub fn uniform(mut self, binding: u32, buffer: &'a wgpu::Buffer) -> Self {
        self.entries.push((
            binding,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        ));
        self
    }

    #[must_use]
    pub fn storage(mut self, binding: u32, buffer: &'a wgpu::Buffer, read_only: bool) -> Self {
        self.entries.push((
            binding,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        ));
        self
    }

    #[must_use]
    pub fn texture(
        mut self,
        binding: u32,
        view: &'a wgpu::TextureView,
        sample_type: wgpu::TextureSampleType,
    ) -> Self {
        self.entries.push((
            binding,
            wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            wgpu::BindingResource::TextureView(view),
        ));
        self
    }

    #[must_use]
    pub fn sampler(mut self, binding: u32, sampler: &'a wgpu::Sampler) -> Self {
        self.entries.push((
            binding,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            wgpu::BindingResource::Sampler(sampler),
        ));
        self
    }

    pub fn build(self) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let visibility = self.visibility.expect("visibility is required");

        let layout_entries: Vec<_> = self
            .entries
            .iter()
            .map(|(binding, ty, _)| wgpu::BindGroupLayoutEntry {
                binding: *binding,
                visibility,
                ty: *ty,
                count: None,
            })
            .collect();

        let layout = self
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &layout_entries,
            });

        let group_entries: Vec<_> = self
            .entries
            .iter()
            .map(|(binding, _, resource)| wgpu::BindGroupEntry {
                binding: *binding,
                resource: resource.clone(),
            })
            .collect();

        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: self.label,
            layout: &layout,
            entries: &group_entries,
        });

        (layout, group)
    }
}

/// Only creates the layout not the `BindGroup`
/// Use when double buffering is necessary
#[derive(Debug)]
pub struct BindGroupLayoutBuilder<'a> {
    device: &'a wgpu::Device,
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    visibility: wgpu::ShaderStages,
    label: Option<&'a str>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub const fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            entries: Vec::new(),
            visibility: wgpu::ShaderStages::all(),
            label: None,
        }
    }

    #[must_use]
    pub const fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = visibility;
        self
    }

    #[must_use]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[must_use]
    pub fn uniform(mut self, binding: u32) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    #[must_use]
    pub fn storage(mut self, binding: u32, read_only: bool) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    #[must_use]
    pub fn texture(mut self, binding: u32, sample_type: wgpu::TextureSampleType) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        });
        self
    }

    #[must_use]
    pub fn sampler(mut self, binding: u32) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self
    }

    pub fn build(self) -> wgpu::BindGroupLayout {
        self.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.label,
                entries: &self.entries,
            })
    }
}

#[derive(Debug)]
pub struct FrameBuffered {
    pub layout: wgpu::BindGroupLayout,
    groups: [wgpu::BindGroup; FRAMES_IN_FLIGHT],
}

impl FrameBuffered {
    /// `bind` is called once per frame slot with that slot's index,
    /// and must return the full entry list for one bind group.
    pub fn new<'a>(
        device: &wgpu::Device,
        layout: wgpu::BindGroupLayout,
        bind: impl Fn(usize) -> Vec<wgpu::BindGroupEntry<'a>>,
    ) -> Self {
        let groups = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("frame_bg"),
                layout: &layout,
                entries: &bind(i),
            })
        });
        Self { layout, groups }
    }

    pub const fn current(&self, frame: usize) -> &wgpu::BindGroup {
        &self.groups[frame % FRAMES_IN_FLIGHT]
    }
}
