/// Create a wgpu::Instance with default parameters
pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    })
}

/// Create a wgpu::Adapter with default parameters
pub fn create_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
    let gpus = instance.enumerate_adapters(wgpu::Backends::PRIMARY);
    for gpu in gpus {
        log::info!("FOUND GPU: {:?}", gpu.get_info());
    }

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    }));
    match adapter {
        Ok(ref adapter) => log::info!("SELECTED GPU: {:?}", adapter.get_info()),
        _ => log::error!("Failed to find adapter"),
    };

    adapter
}

/// Return the wgpu::Device and wgpu::Queue from the adapter
pub fn create_device(
    adapter: &wgpu::Adapter,
) -> impl Future<Output = Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError>> {
    adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        label: None,
        ..Default::default()
    })
}

/// Detect if HDR format is supported by the surface
pub fn detect_hdr_support(caps: &wgpu::SurfaceCapabilities) -> Option<wgpu::TextureFormat> {
    // Prefer Rgba16Float (Metal/macOS), fallback to Rgb10a2Unorm (DX12/Windows)
    caps.formats
        .iter()
        .find(|f| {
            matches!(
                f,
                wgpu::TextureFormat::Rgba16Float | wgpu::TextureFormat::Rgb10a2Unorm
            )
        })
        .copied()
}

/// Select SDR format (sRGB preferred)
fn select_sdr_format(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    caps.formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0])
}

/// Configure the window surface must be called on resize
/// Returns (SurfaceConfiguration, is_hdr_active)
pub fn configure_surface(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    surface: &wgpu::Surface<'_>,
    size: winit::dpi::PhysicalSize<u32>,
    config: &crate::config::EngineConfig,
) -> (wgpu::SurfaceConfiguration, bool) {
    let surface_caps = surface.get_capabilities(&adapter);
    log::info!("FOUND SWAPCHAIN FORMATS: {:?}", surface_caps.formats);

    // Select format based on HDR mode
    let (surface_format, is_hdr) = match config.hdr_mode {
        crate::config::HdrMode::Disabled => {
            (select_sdr_format(&surface_caps), false)
        }
        crate::config::HdrMode::Enabled => {
            if let Some(fmt) = detect_hdr_support(&surface_caps) {
                log::info!("HDR enabled: using format {:?}", fmt);
                (fmt, true)
            } else {
                log::warn!("HDR requested but not supported by display, falling back to SDR");
                (select_sdr_format(&surface_caps), false)
            }
        }
        crate::config::HdrMode::Auto => {
            if let Some(fmt) = detect_hdr_support(&surface_caps) {
                log::info!("HDR auto-detected and enabled: using format {:?}", fmt);
                (fmt, true)
            } else {
                log::info!("HDR not available, using SDR");
                (select_sdr_format(&surface_caps), false)
            }
        }
    };
    log::info!("SELECTED SWAPCHAIN FORMAT: {:?}", surface_format);

    let present_mode = if config.vsync {
        wgpu::PresentMode::Fifo
    } else if surface_caps
        .present_modes
        .contains(&wgpu::PresentMode::Immediate)
    {
        wgpu::PresentMode::Immediate
    } else if surface_caps
        .present_modes
        .contains(&wgpu::PresentMode::Mailbox)
    {
        wgpu::PresentMode::Mailbox
    } else {
        surface_caps.present_modes[0]
    };

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![surface_format],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    (surface_config, is_hdr)
}

/// Create the gbuffer texture
pub fn create_gbuffer_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}

/// Create the depth buffer texture
pub fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    (texture, view)
}
