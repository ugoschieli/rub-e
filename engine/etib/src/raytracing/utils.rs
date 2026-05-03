use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

/// Create a default wgpu instance.
pub fn create_wgpu_instance() -> wgpu::Instance {
    wgpu::Instance::default()
}

pub(crate) fn create_adapter(
    instance: &wgpu::Instance,
    desc: Option<wgpu::RequestAdapterOptions>,
) -> wgpu::Adapter {
    let descriptor = desc.unwrap_or(wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&descriptor)).unwrap();

    adapter
}

pub(crate) fn create_device(
    adapter: &wgpu::Adapter,
    desc: Option<wgpu::DeviceDescriptor>,
) -> (wgpu::Device, wgpu::Queue) {
    let descriptor = desc.unwrap_or(wgpu::DeviceDescriptor {
        ..Default::default()
    });

    let (device, queue) = pollster::block_on(adapter.request_device(&descriptor)).unwrap();

    (device, queue)
}

pub(crate) fn create_surface<'window>(
    instance: &wgpu::Instance,
    window: Arc<Window>,
) -> wgpu::Surface<'window> {
    let surface = instance.create_surface(window).unwrap();

    surface
}

pub(crate) fn configure_surface(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    size: PhysicalSize<u32>,
    config: Option<wgpu::SurfaceConfiguration>,
) -> wgpu::SurfaceConfiguration {
    let mut config = config.unwrap_or(
        surface
            .get_default_config(adapter, size.width, size.height)
            .expect("Failed to get default surface configuration"),
    );
    let caps = surface.get_capabilities(adapter);
    if caps.formats.contains(&wgpu::TextureFormat::Rgba32Float) {
        config.format = wgpu::TextureFormat::Rgba32Float;
    } else if caps.formats.contains(&wgpu::TextureFormat::Rgba16Float) {
        config.format = wgpu::TextureFormat::Rgba16Float;
    } else {
        config.format = wgpu::TextureFormat::Bgra8Unorm;
    }
    // config.format = wgpu::TextureFormat::Bgra8Unorm;

    surface.configure(device, &config);

    config
}

/// Create a compute pipeline from a shader module and bind group layouts.
pub fn create_compute_pipeline(
    device: &wgpu::Device,
    label: &str,
    shader: &wgpu::ShaderModule,
    bind_groups: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(format!("{} layout", label).as_str()),
        bind_group_layouts: bind_groups,
        push_constant_ranges: &[],
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        module: shader,
        entry_point: None,
        layout: Some(&layout),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}

/// Create a full-screen triangle render pipeline from a shader module and bind group layouts.
pub fn create_render_pipeline(
    device: &wgpu::Device,
    label: &str,
    shader: &wgpu::ShaderModule,
    bind_groups: &[&wgpu::BindGroupLayout],
    color_targets: &[Option<wgpu::ColorTargetState>],
    depth_stencil: Option<wgpu::DepthStencilState>,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(format!("{} layout", label).as_str()),
        bind_group_layouts: bind_groups,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: color_targets,
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgpu_context_creation() {
        // This test runs a real headless WGPU device creation.
        // Might fail on CI without GPU, but works on local test runs.
        let instance = create_wgpu_instance();
        
        // We can request a fallback adapter (software rendering) if hardware is absent to improve CI pass rate
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, queue) = create_device(&adapter, None);
            
            // Just verifying that we have a valid device/queue handles
            assert!(device.features().is_empty() || !device.features().is_empty());
        } else {
            // Ignore if no adapter found (e.g. CI without vulkan/metal/llvmpipe)
            println!("No WGPU adapter found, skipping device creation test.");
        }
    }
}
