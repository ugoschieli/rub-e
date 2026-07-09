pub fn create_compute_pipeline(
    device: &wgpu::Device,
    label: &str,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    immediate_size: u32,
    shader: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[bind_group_layout],
        immediate_size,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        module: shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    })
}

pub fn create_compute_pass<'encoder>(
    label: &str,
    encoder: &'encoder mut wgpu::CommandEncoder,
) -> wgpu::ComputePass<'encoder> {
    encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    })
}
