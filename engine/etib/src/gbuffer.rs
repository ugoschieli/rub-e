use crate::utils::wgpu_utils::*;
use etib_core::buffer::VertexBuffer;
use etib_core::{bindgroup::BindGroup, shader::Shader};

pub struct GBuffer {
    pub position_texture: wgpu::Texture,
    pub position_view: wgpu::TextureView,
    pub normal_texture: wgpu::Texture,
    pub normal_view: wgpu::TextureView,
    pub albedo_texture: wgpu::Texture,
    pub albedo_view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

pub fn create_gbuffer(device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) -> GBuffer {
    let (position_texture, position_view) = create_gbuffer_texture(
        device,
        size.width,
        size.height,
        wgpu::TextureFormat::Rgba32Float,
        "Position Texture",
    );

    let (normal_texture, normal_view) = create_gbuffer_texture(
        device,
        size.width,
        size.height,
        wgpu::TextureFormat::Rgba16Float,
        "Normal Texture",
    );

    let (albedo_texture, albedo_view) = create_gbuffer_texture(
        device,
        size.width,
        size.height,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        "Albedo Texture",
    );

    let (depth_texture, depth_view) = create_depth_texture(&device, size.width, size.height);

    GBuffer {
        position_texture,
        position_view,
        normal_texture,
        normal_view,
        albedo_texture,
        albedo_view,
        depth_texture,
        depth_view,
    }
}

pub fn create_gbuffer_pipeline<T: bytemuck::Pod>(
    device: &wgpu::Device,
    camera_uniform: &BindGroup,
    vertex_buffer: &VertexBuffer<T>,
) -> wgpu::RenderPipeline {
    let geometry_shader = Shader::new(
        include_str!("./shaders/geometry.wgsl"),
        device,
        Some("GBuffer Shader"),
    );

    let geometry_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Geometry Pipeline Layout"),
        bind_group_layouts: &[&camera_uniform.layout],
        push_constant_ranges: &[],
    });

    let geometry_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Geometry Pipeline"),
        layout: Some(&geometry_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &geometry_shader.module,
            entry_point: Some("vs_main"),
            buffers: &[vertex_buffer.layout.clone()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &geometry_shader.module,
            entry_point: Some("fs_main"),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    });

    geometry_pipeline
}
