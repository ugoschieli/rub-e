#[derive(Debug, Default)]
pub struct RenderPassBuilder<'a, 'tex> {
    label: Option<&'a str>,
    depth_stencil_view: Option<&'tex wgpu::TextureView>,
    depth_ops: Option<wgpu::Operations<f32>>,
    stencil_ops: Option<wgpu::Operations<u32>>,
    targets: Vec<(&'tex wgpu::TextureView, wgpu::Operations<wgpu::Color>)>,
}

impl<'a, 'tex> RenderPassBuilder<'a, 'tex> {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[must_use]
    pub fn target(
        mut self,
        view: &'tex wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        store: wgpu::StoreOp,
    ) -> Self {
        self.targets.push((view, wgpu::Operations { load, store }));
        self
    }

    #[must_use]
    pub const fn depth_stencil_view(mut self, view: &'tex wgpu::TextureView) -> Self {
        self.depth_stencil_view = Some(view);
        self
    }

    #[must_use]
    pub const fn depth_ops(mut self, load: wgpu::LoadOp<f32>, store: wgpu::StoreOp) -> Self {
        self.depth_ops = Some(wgpu::Operations { load, store });
        self
    }

    #[must_use]
    pub const fn stencil_ops(mut self, operations: wgpu::Operations<u32>) -> Self {
        self.stencil_ops = Some(operations);
        self
    }

    pub fn build(self, encoder: &'_ mut wgpu::CommandEncoder) -> wgpu::RenderPass<'_> {
        let depth_stencil_attachment = if self.depth_stencil_view.is_some() {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: self
                    .depth_stencil_view
                    .expect("The DepthStencil Texture is missing"),
                depth_ops: self.depth_ops,
                stencil_ops: self.stencil_ops,
            })
        } else {
            None
        };

        let color_attachments = self
            .targets
            .into_iter()
            .map(|(view, ops)| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    ops,
                    depth_slice: None,
                    resolve_target: None,
                })
            })
            .collect::<Vec<Option<wgpu::RenderPassColorAttachment>>>();

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: self.label,
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        })
    }
}
