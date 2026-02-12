/// Manages the Hierarchical Z-Buffer texture
pub struct HiZBuffer {
    pub texture: wgpu::Texture,
    pub full_view: wgpu::TextureView,
    pub mip_views: Vec<wgpu::TextureView>,
    pub width: u32,
    pub height: u32,
    pub mips: u32,
}

impl HiZBuffer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        // Calculate number of mips to get down to 1x1
        let mips = ((width.max(height) as f32).log2().floor() as u32) + 1;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Hi-Z Buffer"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mips,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let full_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Hi-Z Full View"),
            ..Default::default()
        });

        let mut mip_views = Vec::with_capacity(mips as usize);
        for i in 0..mips {
            mip_views.push(texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&format!("Hi-Z Mip {}", i)),
                base_mip_level: i,
                mip_level_count: Some(1),
                ..Default::default()
            }));
        }

        Self {
            texture,
            full_view,
            mip_views,
            width,
            height,
            mips,
        }
    }
}

/// Pipeline to generate the Hi-Z buffer mips
pub struct OcclusionPass {
    downsample_pipeline: wgpu::ComputePipeline,
    copy_pipeline: wgpu::ComputePipeline,
    bind_group_layout_downsample: wgpu::BindGroupLayout,
    bind_group_layout_copy: wgpu::BindGroupLayout,
}

impl OcclusionPass {
    pub fn new(device: &wgpu::Device) -> Self {
        // Layout for Downsample (R32F -> R32F)
        let bind_group_layout_downsample =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Hi-Z Downsample Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        // Layout for Copy (Depth32F -> R32F)
        let bind_group_layout_copy =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Hi-Z Copy Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth, // Different sample type for depth
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hi-Z Downsample Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/hiz.wgsl").into()),
        });

        let copy_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hi-Z Copy Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/copy_depth.wgsl").into()),
        });

        let downsample_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hi-Z Downsample Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout_downsample],
            push_constant_ranges: &[],
        });

        let copy_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hi-Z Copy Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout_copy],
            push_constant_ranges: &[],
        });

        let downsample_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Hi-Z Downsample Pipeline"),
                layout: Some(&downsample_layout),
                module: &downsample_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let copy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Hi-Z Copy Pipeline"),
            layout: Some(&copy_layout),
            module: &copy_shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            downsample_pipeline,
            copy_pipeline,
            bind_group_layout_downsample,
            bind_group_layout_copy,
        }
    }

    pub fn generate(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        hiz: &HiZBuffer,
        depth_view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Hi-Z Generation Pass"),
            timestamp_writes: None,
        });

        // Pass 0: Depth Buffer -> Hi-Z Mip 0 (Copy)
        pass.set_pipeline(&self.copy_pipeline);
        {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Hi-Z Copy Bind Group"),
                layout: &self.bind_group_layout_copy,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(depth_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&hiz.mip_views[0]),
                    },
                ],
            });
            pass.set_bind_group(0, &bind_group, &[]);

            let dst_width = hiz.width;
            let dst_height = hiz.height;
            let x_groups = (dst_width + 15) / 16;
            let y_groups = (dst_height + 15) / 16;
            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }

        // Subsequent Mips (Downsample)
        pass.set_pipeline(&self.downsample_pipeline);
        for i in 0..(hiz.mips - 1) {
            let input_view = &hiz.mip_views[i as usize];
            let output_view = &hiz.mip_views[(i + 1) as usize];

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Hi-Z Downsample Bind Group {}->{}", i, i + 1)),
                layout: &self.bind_group_layout_downsample,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(input_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(output_view),
                    },
                ],
            });

            pass.set_bind_group(0, &bind_group, &[]);

            // Calculate dispatch size
            let dst_width = (hiz.width >> (i + 1)).max(1);
            let dst_height = (hiz.height >> (i + 1)).max(1);

            // Workgroup size is 16x16
            let x_groups = (dst_width + 15) / 16;
            let y_groups = (dst_height + 15) / 16;

            pass.dispatch_workgroups(x_groups, y_groups, 1);
        }
    }
}
