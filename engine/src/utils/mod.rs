pub mod bindgroup;
pub mod pipeline;

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::constants::{DEPTH_FORMAT, FRAMES_IN_FLIGHT};

pub fn create_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env())
}

pub fn create_adapter(instance: &wgpu::Instance) -> wgpu::Adapter {
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("Failed to obtain an adapter")
}

pub fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    let adapter_limits = adapter.limits();
    log::info!("{adapter_limits:#?}");

    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::default(),
        required_limits: adapter_limits,
        experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
        ..Default::default()
    }))
    .expect("Failed to obtain a device")
}

pub fn create_surface(
    instance: &wgpu::Instance,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    window: impl Into<wgpu::SurfaceTarget<'static>>,
    size: PhysicalSize<u32>,
) -> (wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
    let surface = instance
        .create_surface(window)
        .expect("Failed to create a surface");

    let mut surface_config = surface
        .get_default_config(adapter, size.width, size.height)
        .expect("The surface isn't supported by this adapter");
    surface_config.desired_maximum_frame_latency = u32::try_from(FRAMES_IN_FLIGHT).unwrap();
    surface_config.present_mode = wgpu::PresentMode::AutoVsync;
    surface_config.present_mode = wgpu::PresentMode::Immediate;
    log::info!("{surface_config:?}");

    surface.configure(device, &surface_config);

    (surface, surface_config)
}

pub fn create_buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

pub fn create_buffer_init<T: NoUninit>(
    device: &wgpu::Device,
    label: &str,
    usage: wgpu::BufferUsages,
    contents: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        usage,
        contents: bytemuck::cast_slice(contents),
    })
}

/// Build a `main`-entry compute pipeline over a single bind group layout from an
/// already-created shader module (e.g. via `include_wgsl!`).
pub fn create_compute_pipeline_from_module(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: &wgpu::BindGroupLayout,
    module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        module,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}

pub fn create_depth_texture(
    device: &wgpu::Device,
    size: PhysicalSize<u32>,
) -> (wgpu::Texture, wgpu::TextureView) {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ETIB Depth Texture"),
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        dimension: wgpu::TextureDimension::D2,
        mip_level_count: 1,
        sample_count: 1,
        view_formats: &[],
    });

    let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("ETIB Depth Texture View"),
        ..Default::default()
    });

    (depth_texture, depth_texture_view)
}

pub fn create_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ETIB Command Encoder"),
    })
}

pub fn create_compute_pass(encoder: &'_ mut wgpu::CommandEncoder) -> wgpu::ComputePass<'_> {
    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ETIB Compute Pass"),
        timestamp_writes: None,
    })
}

pub fn create_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    current_surface_texture_view: &wgpu::TextureView,
    depth_texture_view: &wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ETIB Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: current_surface_texture_view,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
            resolve_target: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}

/// Like [`create_render_pass`] but preserves the existing color and depth
/// contents (`LoadOp::Load`) so the pass composites over earlier renderers.
pub fn create_loading_render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    current_surface_texture_view: &wgpu::TextureView,
    depth_texture_view: &wgpu::TextureView,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("ETIB Loading Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: current_surface_texture_view,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
            resolve_target: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_texture_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    })
}
