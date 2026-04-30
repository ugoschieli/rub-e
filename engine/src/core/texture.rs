/// A texture
#[derive(Debug, Clone)]
pub struct Texture {
    /// The underlying wgpu::Texture
    pub texture: wgpu::Texture,
    /// The underlying wgpu::TextureView
    pub view: wgpu::TextureView,
}

impl Texture {
    /// Create a new texture
    pub fn new(
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        usages: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: usages,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}
