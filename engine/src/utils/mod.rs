use bytemuck::NoUninit;
use wgpu::util::DeviceExt;

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

pub fn create_compute_pass(encoder: &'_ mut wgpu::CommandEncoder) -> wgpu::ComputePass<'_> {
    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("ETIB Compute Pass"),
        timestamp_writes: None,
    })
}
