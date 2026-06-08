use crate::constants::DEPTH_FORMAT;

pub struct RenderPipelineBuilder<'a> {
    device: &'a wgpu::Device,
    bindgroups: Vec<&'a wgpu::BindGroupLayout>,
    vertex_shader: Option<&'a wgpu::ShaderModule>,
    fragment_shader: Option<&'a wgpu::ShaderModule>,
    vertex_buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    color_targets: Vec<Option<wgpu::ColorTargetState>>,
    depth_stencil: Option<wgpu::DepthStencilState>,
    cull_mode: Option<wgpu::Face>,
    label: Option<&'a str>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device,
            bindgroups: vec![],
            vertex_shader: None,
            fragment_shader: None,
            vertex_buffers: vec![],
            color_targets: vec![],
            depth_stencil: None,
            cull_mode: None,
            label: None,
        }
    }

    pub fn bind_group(mut self, layout: &'a wgpu::BindGroupLayout) -> Self {
        self.bindgroups.push(layout);
        self
    }

    pub fn vertex(
        mut self,
        module: &'a wgpu::ShaderModule,
        buffers: &[wgpu::VertexBufferLayout<'a>],
    ) -> Self {
        self.vertex_shader = Some(module);
        self.vertex_buffers = buffers.to_vec();
        self
    }

    pub fn fragment(
        mut self,
        module: &'a wgpu::ShaderModule,
        targets: &[Option<wgpu::ColorTargetState>],
    ) -> Self {
        self.fragment_shader = Some(module);
        self.color_targets = targets.to_vec();
        self
    }

    pub fn depth_stencil(mut self, state: wgpu::DepthStencilState) -> Self {
        self.depth_stencil = Some(state);
        self
    }

    pub fn with_depth_test(mut self) -> Self {
        self.depth_stencil = Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        self
    }

    pub fn with_backface_culling(mut self) -> Self {
        self.cull_mode = Some(wgpu::Face::Back);
        self
    }

    pub fn with_frontface_culling(mut self) -> Self {
        self.cull_mode = Some(wgpu::Face::Front);
        self
    }

    pub fn build(self) -> wgpu::RenderPipeline {
        let vs = self.vertex_shader.expect("vertex shader required");
        let fs = self.fragment_shader.expect("fragment shader required");

        let layout = if self.bindgroups.len() == 0 {
            None
        } else {
            Some(
                &self
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: self.label,
                        bind_group_layouts: &self
                            .bindgroups
                            .iter()
                            .map(|layout| Some(*layout))
                            .collect::<Vec<_>>(),
                        immediate_size: 0,
                    }),
            )
        };

        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: self.label,
                layout,
                vertex: wgpu::VertexState {
                    module: &vs,
                    entry_point: Some("vs_main"),
                    buffers: &self.vertex_buffers,
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs,
                    entry_point: Some("fs_main"),
                    targets: &self.color_targets,
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: self.cull_mode,
                    ..Default::default()
                },
                depth_stencil: self.depth_stencil,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }
}
