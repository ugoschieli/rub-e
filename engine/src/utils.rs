use bytemuck::NoUninit;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub fn create_instance(event_loop: &ActiveEventLoop) -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
        Box::new(event_loop.owned_display_handle()),
    ))
}

pub fn create_adapter(instance: &wgpu::Instance) -> wgpu::Adapter {
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("Failed to obtain an adapter")
}

pub fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::default(),
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
    surface_config.desired_maximum_frame_latency = 0;
    surface_config.present_mode = wgpu::PresentMode::AutoVsync;
    //surface_config.present_mode = wgpu::PresentMode::Immediate;
    log::info!("{surface_config:?}");

    surface.configure(device, &surface_config);

    (surface, surface_config)
}

pub fn create_buffer<T: NoUninit>(
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

pub fn create_bind_group(
    device: &wgpu::Device,
    camera_buffer: &wgpu::Buffer,
    cubes_buffer: &wgpu::Buffer,
    face_matrices_buffer: &wgpu::Buffer,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ETIB Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0, // Camera
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1, // Cubes
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2, // Face Rotation Matrices
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ETIB Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0, // Camera
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1, // Cubes
                resource: cubes_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2, // Face Rotation Matrices
                resource: face_matrices_buffer.as_entire_binding(),
            },
        ],
    });

    (bind_group, bind_group_layout)
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    surface_config: &wgpu::SurfaceConfiguration,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::include_wgsl!("./shaders.wgsl"));

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ETIB Pipeline Layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("ETIB Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: None,
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: None,
            targets: &[Some(surface_config.format.into())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            stencil: wgpu::StencilState::default(),
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
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

pub fn get_current_surface_texture(
    surface: &wgpu::Surface,
) -> Option<(wgpu::SurfaceTexture, wgpu::TextureView)> {
    let current_surface_texture = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Occluded => {
            return None;
        }
        wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
        _ => panic!("Failed to obtain the current surface texture"),
    };

    let current_surface_texture_view =
        current_surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("ETIB Current Surface Texture View"),
                ..Default::default()
            });

    Some((current_surface_texture, current_surface_texture_view))
}

pub fn create_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ETIB Command Encoder"),
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
