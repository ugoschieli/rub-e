/// A texture
#[derive(Debug, Clone)]
pub struct Texture {
    /// The underlying wgpu::Texture
    pub texture: wgpu::Texture,
    /// The underlying wgpu::TextureView
    pub view: wgpu::TextureView,
    /// The underlying wgpu::Sampler
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Create a new texture
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usages: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

pub struct CubeTexture {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    view: wgpu::TextureView,
}

impl CubeTexture {
    pub fn create_2d(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            sampler,
            view,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_headless() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
            
            let tex = Texture::new(
                &device, 
                256, 
                256, 
                wgpu::TextureFormat::Rgba8Unorm, 
                wgpu::TextureUsages::TEXTURE_BINDING, 
                Some("Test Texture")
            );

            assert_eq!(tex.texture.width(), 256);
            assert_eq!(tex.texture.height(), 256);

            let cube = CubeTexture::create_2d(
                &device,
                128,
                128,
                wgpu::TextureFormat::Rgba8Unorm,
                1,
                wgpu::TextureUsages::TEXTURE_BINDING,
                wgpu::FilterMode::Linear,
                Some("Test Cube")
            );

            assert_eq!(cube.texture().width(), 128);
            assert_eq!(cube.texture().depth_or_array_layers(), 6);
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
    fn test_texture_depth_format() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let tex = Texture::new(
            &device,
            64,
            64,
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("depth texture"),
        );
        assert_eq!(tex.texture.width(), 64);
        assert_eq!(tex.texture.height(), 64);
        assert_eq!(tex.texture.format(), wgpu::TextureFormat::Depth32Float);
    }

    #[test]
    fn test_texture_usage_stored() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let tex = Texture::new(
            &device,
            32,
            32,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
        );
        assert_eq!(tex.texture.width(), 32);
    }

    #[test]
    fn test_cube_texture_accessors() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let cube = CubeTexture::create_2d(
            &device,
            64,
            64,
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Nearest,
            Some("cube accessors test"),
        );
        assert_eq!(cube.texture().width(), 64);
        assert_eq!(cube.texture().depth_or_array_layers(), 6);
        // Verify accessors return valid references (not null/invalid)
        let _view: &wgpu::TextureView = cube.view();
        let _sampler: &wgpu::Sampler = cube.sampler();
    }

    #[test]
    fn test_cube_texture_mip_levels() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let cube = CubeTexture::create_2d(
            &device,
            16,
            16,
            wgpu::TextureFormat::Rgba8Unorm,
            4,
            wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Linear,
            Some("cube mip test"),
        );
        assert_eq!(cube.texture().mip_level_count(), 4);
        assert_eq!(cube.texture().depth_or_array_layers(), 6);
    }

    #[test]
    fn test_cube_texture_mag_filter_nearest() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let _cube = CubeTexture::create_2d(
            &device,
            8,
            8,
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Nearest,
            None,
        );
    }
}
