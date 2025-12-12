use std::clone::Clone;
use std::sync::Arc;

use winit::keyboard::PhysicalKey;
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::camera::{Camera, CameraController};
use crate::core::bindgroup::*;
use crate::core::shader::Shader;
use crate::core::texture::Texture;
use crate::core::time::TimeState;
use crate::gbuffer;
use crate::wgpu_utils;

pub struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // G-buffer textures
    gbuffer: gbuffer::GBuffer,

    // Geometry pass
    geometry_pipeline: wgpu::RenderPipeline,

    // Lighting pass (ray tracing compute)
    raytracing_pipeline: wgpu::ComputePipeline,
    raytracing_bind_group: BindGroup,
    raytracing_output_texture: Texture,

    // Final blit pass
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: BindGroup,

    // Camera
    camera: Camera,
    camera_uniform: BindGroup,
    camera_controller: CameraController,

    // Model
    model: Model,

    // Mouse state
    mouse_pressed: bool,
    last_mouse_pos: (f64, f64),

    // Time and FPS tracking
    pub time: TimeState,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu_utils::create_instance();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = wgpu_utils::create_adapter(&instance, &surface).unwrap();
        let (device, queue) = wgpu_utils::create_device(&adapter).await.unwrap();

        let surface_config =
            wgpu_utils::configure_surface(&adapter, &device, &surface, size, false);

        let gbuffer = gbuffer::create_gbuffer(&device, size);

        // Camera setup
        let camera = Camera::new(glam::Vec3::new(0.0, 2.0, 5.0));
        let camera_controller = CameraController::new(5.0, 0.002);
        let camera_bindgroup = BindGroupBuilder::new()
            .add_uniform_buffer(
                &device,
                0,
                bytemuck::bytes_of(&camera.to_uniform(size.width as f32 / size.height as f32)),
                wgpu::ShaderStages::VERTEX,
            )
            .build(&device, Some("Camera Bind Group"));

        let geometry_pipeline = gbuffer::create_gbuffer_pipeline(&device, &camera_bindgroup);

        // Ray tracing output texture
        let raytracing_output_texture = Texture::new(
            &device,
            size,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            Some("Raytracing Output Texture"),
        );

        let raytracing_bind_group = BindGroupBuilder::new()
            .add_texture(
                0,
                gbuffer.position_view.clone(),
                wgpu::ShaderStages::COMPUTE,
                wgpu::TextureSampleType::Float { filterable: false },
            )
            .add_texture(
                1,
                gbuffer.normal_view.clone(),
                wgpu::ShaderStages::COMPUTE,
                wgpu::TextureSampleType::Float { filterable: false },
            )
            .add_texture(
                2,
                gbuffer.albedo_view.clone(),
                wgpu::ShaderStages::COMPUTE,
                wgpu::TextureSampleType::Float { filterable: false },
            )
            .add_uniform_buffer(
                &device,
                3,
                bytemuck::bytes_of(&camera.to_uniform(size.width as f32 / size.height as f32)),
                wgpu::ShaderStages::COMPUTE,
            )
            .add_storage_texture(
                4,
                raytracing_output_texture.view.clone(),
                wgpu::ShaderStages::COMPUTE,
            )
            .build(&device, Some("Raytracing Bind Group"));

        let raytracing_shader = Shader::new(
            include_str!("./shaders/raytracing.wgsl").into(),
            &device,
            Some("Raytracing Shader"),
        );

        let raytracing_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ray Tracing Pipeline Layout"),
                bind_group_layouts: &[&raytracing_bind_group.layout],
                push_constant_ranges: &[],
            });

        let raytracing_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Ray Tracing Pipeline"),
                layout: Some(&raytracing_pipeline_layout),
                module: &raytracing_shader.module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        // Blit pass to display ray traced result
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let blit_bind_group = BindGroupBuilder::new()
            .add_texture(
                0,
                raytracing_output_texture.view.clone(),
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .add_sampler(
                1,
                sampler.clone(),
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(&device, Some("Blit Bind Group"));

        let blit_shader = Shader::new(
            include_str!("./shaders/blit.wgsl").into(),
            &device,
            Some("Blit Shader"),
        );

        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&blit_bind_group.layout],
            push_constant_ranges: &[],
        });

        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader.module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Load model (cube with plane beneath it)
        let model = Model::create_cube_with_plane(&device);

        Self {
            window,
            surface,
            device,
            queue,
            config: surface_config,
            size,
            gbuffer,
            geometry_pipeline,
            raytracing_pipeline,
            raytracing_bind_group,
            raytracing_output_texture,
            blit_pipeline,
            blit_bind_group,
            camera,
            camera_controller,
            camera_uniform: camera_bindgroup,
            model,
            mouse_pressed: false,
            last_mouse_pos: (0.0, 0.0),
            time: TimeState::new(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Recreate G-buffer textures
            self.gbuffer = gbuffer::create_gbuffer(&self.device, self.size);

            // Recreate ray tracing output texture
            self.raytracing_output_texture = Texture::new(
                &self.device,
                self.size,
                wgpu::TextureFormat::Rgba8Unorm,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                Some("Raytracing Output Texture"),
            );

            self.raytracing_bind_group = BindGroupBuilder::new()
                .add_texture(
                    0,
                    self.gbuffer.position_view.clone(),
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Float { filterable: false },
                )
                .add_texture(
                    1,
                    self.gbuffer.normal_view.clone(),
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Float { filterable: false },
                )
                .add_texture(
                    2,
                    self.gbuffer.albedo_view.clone(),
                    wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Float { filterable: false },
                )
                .add_uniform_buffer(
                    &self.device,
                    3,
                    bytemuck::bytes_of(
                        &self
                            .camera
                            .to_uniform(self.size.width as f32 / self.size.height as f32),
                    ),
                    wgpu::ShaderStages::COMPUTE,
                )
                .add_storage_texture(
                    4,
                    self.raytracing_output_texture.view.clone(),
                    wgpu::ShaderStages::COMPUTE,
                )
                .build(&self.device, Some("Raytracing Bind Group"));

            // Recreate blit bind group
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            self.blit_bind_group = BindGroupBuilder::new()
                .add_texture(
                    0,
                    self.raytracing_output_texture.view.clone(),
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::TextureSampleType::Float { filterable: true },
                )
                .add_sampler(
                    1,
                    sampler.clone(),
                    wgpu::SamplerBindingType::Filtering,
                    wgpu::ShaderStages::FRAGMENT,
                )
                .build(&self.device, Some("Blit Bind Group"));
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x, position.y);
                if self.mouse_pressed {
                    let dx = new_pos.0 - self.last_mouse_pos.0;
                    let dy = new_pos.1 - self.last_mouse_pos.1;
                    self.camera_controller
                        .process_mouse(dx, dy, &mut self.camera);
                }
                self.last_mouse_pos = new_pos;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.time.tick();

        self.camera_controller
            .update_camera(&mut self.camera, self.time.dt);

        let camera_data = self
            .camera
            .to_uniform(self.size.width as f32 / self.size.height as f32);
        self.camera_uniform
            .write_buffer(&self.queue, 0, bytemuck::bytes_of(&camera_data));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Geometry pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Geometry Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.gbuffer.position_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.gbuffer.normal_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.gbuffer.albedo_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.5,
                                g: 0.5,
                                b: 0.5,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.gbuffer.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.geometry_pipeline);
            render_pass.set_bind_group(0, &self.camera_uniform.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.model.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(self.model.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.model.num_indices, 0, 0..1);
        }

        // Ray tracing compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Tracing Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.raytracing_pipeline);
            compute_pass.set_bind_group(0, &self.raytracing_bind_group.bind_group, &[]);

            // Dispatch compute shader (8x8 workgroup size)
            let workgroup_count_x = (self.size.width + 7) / 8;
            let workgroup_count_y = (self.size.height + 7) / 8;
            compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
        }

        // Blit pass to display ray traced result
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.blit_bind_group.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
