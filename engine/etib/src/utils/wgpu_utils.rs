/// Create a wgpu::Instance with default parameters
pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    })
}

/// Create a wgpu::Adapter with default parameters
#[cfg(not(tarpaulin_include))]
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
        label: Some("ETIB: device"),
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
pub(crate) fn select_sdr_format(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    caps.formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0])
}

/// Configure the window surface must be called on resize
/// Returns (SurfaceConfiguration, is_hdr_active)
#[cfg(not(tarpaulin_include))]
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
        crate::config::HdrMode::Disabled => (select_sdr_format(&surface_caps), false),
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

    #[test]
    fn test_create_instance() {
        // Verify create_instance() returns an Instance without panicking
        let _instance = create_instance();
    }

    #[test]
    fn test_create_device_via_create_instance() {
        let instance = create_instance();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        if let Ok(adapter) = adapter {
            // create_device is the wrapper we need to cover
            let result = pollster::block_on(create_device(&adapter));
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_detect_hdr_support_rgba16float() {
        let caps = wgpu::SurfaceCapabilities {
            formats: vec![
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Rgba16Float,
            ],
            present_modes: vec![],
            alpha_modes: vec![],
            usages: wgpu::TextureUsages::empty(),
        };
        assert_eq!(detect_hdr_support(&caps), Some(wgpu::TextureFormat::Rgba16Float));
    }

    #[test]
    fn test_detect_hdr_support_rgb10a2() {
        // Rgb10a2Unorm should also be detected as HDR
        let caps = wgpu::SurfaceCapabilities {
            formats: vec![wgpu::TextureFormat::Rgb10a2Unorm],
            present_modes: vec![],
            alpha_modes: vec![],
            usages: wgpu::TextureUsages::empty(),
        };
        assert_eq!(detect_hdr_support(&caps), Some(wgpu::TextureFormat::Rgb10a2Unorm));
    }

    #[test]
    fn test_detect_hdr_support_none() {
        let caps = wgpu::SurfaceCapabilities {
            formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
            present_modes: vec![],
            alpha_modes: vec![],
            usages: wgpu::TextureUsages::empty(),
        };
        assert_eq!(detect_hdr_support(&caps), None);
    }

    #[test]
    fn test_wgpu_utils_headless() {
        let instance = create_instance();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, _) = pollster::block_on(create_device(&adapter)).unwrap();
            
            let (gbuffer_tex, _) = create_gbuffer_texture(&device, 100, 100, wgpu::TextureFormat::Rgba8Unorm, "test");
            assert_eq!(gbuffer_tex.width(), 100);
            
            let (depth_tex, _) = create_depth_texture(&device, 100, 100);
            assert_eq!(depth_tex.width(), 100);
        } else {
            println!("No adapter found, skipping headless test.");
        }
    }

    #[test]
    fn test_create_gbuffer_texture_dimensions() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let (tex, view) = create_gbuffer_texture(
            &device, 256, 128, wgpu::TextureFormat::Rgba8Unorm, "gbuf_test"
        );
        assert_eq!(tex.width(), 256);
        assert_eq!(tex.height(), 128);
        assert_eq!(tex.format(), wgpu::TextureFormat::Rgba8Unorm);
        // view should be a valid view — just check it compiles by dropping it
        drop(view);
    }

    #[test]
    fn test_create_depth_texture_dimensions() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let (tex, _view) = create_depth_texture(&device, 64, 32);
        assert_eq!(tex.width(), 64);
        assert_eq!(tex.height(), 32);
        assert_eq!(tex.format(), wgpu::TextureFormat::Depth32Float);
    }

    // -----------------------------------------------------------------------
    // select_sdr_format — covers lines 58-63
    // -----------------------------------------------------------------------

    fn make_caps(formats: Vec<wgpu::TextureFormat>, present_modes: Vec<wgpu::PresentMode>) -> wgpu::SurfaceCapabilities {
        wgpu::SurfaceCapabilities {
            formats,
            present_modes,
            alpha_modes: vec![wgpu::CompositeAlphaMode::Auto],
            usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
        }
    }

    #[test]
    fn test_select_sdr_format_prefers_srgb() {
        let caps = make_caps(
            vec![wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba8UnormSrgb],
            vec![],
        );
        let fmt = select_sdr_format(&caps);
        assert_eq!(fmt, wgpu::TextureFormat::Rgba8UnormSrgb, "should prefer sRGB format");
    }

    #[test]
    fn test_select_sdr_format_fallback_to_first() {
        // No sRGB formats — must fall back to caps.formats[0]
        let caps = make_caps(
            vec![wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureFormat::Rgba16Float],
            vec![],
        );
        let fmt = select_sdr_format(&caps);
        assert_eq!(fmt, wgpu::TextureFormat::Rgba8Unorm, "should fall back to formats[0]");
    }

    // -----------------------------------------------------------------------
    // configure_surface present_mode logic (lines 102-116) — test independently
    // The actual configure_surface() requires a real wgpu::Surface, but we can
    // inline the same decision logic here to verify every branch.
    // -----------------------------------------------------------------------

    fn pick_present_mode(
        vsync: bool,
        present_modes: &[wgpu::PresentMode],
    ) -> wgpu::PresentMode {
        // Mirrors the logic inside configure_surface
        if vsync {
            wgpu::PresentMode::Fifo
        } else if present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else if present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            present_modes[0]
        }
    }

    #[test]
    fn test_present_mode_vsync() {
        let mode = pick_present_mode(
            true,
            &[wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox],
        );
        assert_eq!(mode, wgpu::PresentMode::Fifo);
    }

    #[test]
    fn test_present_mode_immediate() {
        let mode = pick_present_mode(
            false,
            &[wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox],
        );
        assert_eq!(mode, wgpu::PresentMode::Immediate);
    }

    #[test]
    fn test_present_mode_mailbox_fallback() {
        let mode = pick_present_mode(
            false,
            &[wgpu::PresentMode::Mailbox, wgpu::PresentMode::Fifo],
        );
        assert_eq!(mode, wgpu::PresentMode::Mailbox);
    }

    #[test]
    fn test_present_mode_last_resort_fallback() {
        let mode = pick_present_mode(
            false,
            &[wgpu::PresentMode::FifoRelaxed],
        );
        assert_eq!(mode, wgpu::PresentMode::FifoRelaxed);
    }
}
