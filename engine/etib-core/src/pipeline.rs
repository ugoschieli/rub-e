use super::buffer::VertexBuffer;
use super::shader::Shader;

/// The render pipeline
pub struct Pipeline {
    /// The raw wgpu::RenderPipeline
    pub pipeline: wgpu::RenderPipeline,
    /// The wgpu::PipelineLayout
    pub layout: wgpu::PipelineLayout,
}

impl Pipeline {
    /// Create a new pipeline with backface culling and depth testing
    pub fn new<T: bytemuck::Pod>(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader: &Shader,
        surface_config: &wgpu::SurfaceConfiguration,
        vertex_buffer: &VertexBuffer<T>,
    ) -> Pipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer.layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(surface_config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Pipeline {
            layout: pipeline_layout,
            pipeline,
        }
    }

    /// Create a new pipeline with custom vertex buffer layouts (for instancing)
    pub fn new_with_layouts(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader: &Shader,
        surface_config: &wgpu::SurfaceConfiguration,
        vertex_buffer_layouts: &[wgpu::VertexBufferLayout],
    ) -> Pipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                buffers: vertex_buffer_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(surface_config.format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Pipeline {
            layout: pipeline_layout,
            pipeline,
        }
    }

    /// Create a new pipeline with optional backface culling and depth testing
    pub fn new_v2(
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_buffers: &[wgpu::VertexBufferLayout],
        shader: &Shader,
        surface_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        topology: wgpu::PrimitiveTopology,
        label: Option<&str>,
    ) -> Pipeline {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        let mut descriptor = wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                buffers: vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(surface_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        };

        if let Some(depth_format) = depth_format {
            descriptor.depth_stencil = Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            });
        }

        let pipeline = device.create_render_pipeline(&descriptor);

        Pipeline { layout, pipeline }
    }
}
