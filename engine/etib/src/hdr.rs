use std::io::Cursor;

use image::DynamicImage;
use image::codecs::hdr::HdrDecoder;
use wgpu::Operations;

use etib_core::bindgroup;
use etib_core::pipeline;
use etib_core::shader;
use etib_core::texture;

/// Tonemapping mode for HDR pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonemappingMode {
    /// SDR mode with ACES tonemapping
    Sdr,
    /// HDR mode with PQ tonemapping
    Hdr,
}

/// Uniform data for tonemapping shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TonemapUniforms {
    peak_brightness_nits: f32,
    mode: u32, // 0 = SDR, 1 = HDR
    _padding: [f32; 2],
}

/// Owns the render texture and controls tonemapping
pub struct HdrPipeline {
    pipeline: pipeline::Pipeline,
    bind_group: bindgroup::BindGroup,
    texture: texture::Texture,
    tonemap_uniforms: wgpu::Buffer,
    tonemap_bind_group: bindgroup::BindGroup,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    mode: TonemappingMode,
}

impl HdrPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        mode: TonemappingMode,
        peak_brightness_nits: f32,
    ) -> Self {
        let width = config.width;
        let height = config.height;

        // We could use `Rgba32Float`, but that requires some extra
        // features to be enabled for rendering.
        let format = wgpu::TextureFormat::Rgba16Float;

        let texture = texture::Texture::new(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("HDR texture"),
        );

        let bind_group = bindgroup::BindGroupBuilder::new()
            .add_texture(
                0,
                texture.view.clone(),
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .add_sampler(
                1,
                texture.sampler.clone(),
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(device, Some("HDR layout"));

        // Create uniform buffer for tonemapping parameters
        let uniforms = TonemapUniforms {
            peak_brightness_nits,
            mode: if mode == TonemappingMode::Hdr { 1 } else { 0 },
            _padding: [0.0; 2],
        };

        let tonemap_uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tonemap Uniforms"),
            size: std::mem::size_of::<TonemapUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });

        {
            let mut buffer_view = tonemap_uniforms.slice(..).get_mapped_range_mut();
            buffer_view.copy_from_slice(bytemuck::bytes_of(&uniforms));
        }
        tonemap_uniforms.unmap();

        // Create bind group for tonemapping uniforms (group 1)
        let tonemap_bind_group = bindgroup::BindGroupBuilder::new()
            .add_uniform_buffer(
                device,
                0,
                bytemuck::bytes_of(&uniforms),
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(device, Some("Tonemap Uniforms"));

        let shader_str = include_str!("./shaders/hdr.wgsl");
        let shader = shader::Shader::new(shader_str, device, Some("HDR shader"));

        let pipeline = pipeline::Pipeline::new_v2(
            device,
            &[&bind_group.layout, &tonemap_bind_group.layout],
            // We'll use some math to generate the vertex data in
            // the shader, so we don't need any vertex buffers
            &[],
            &shader,
            config.format,
            None,
            wgpu::PrimitiveTopology::TriangleList,
            Some("Hdr pipeline"),
        );

        Self {
            pipeline,
            bind_group,
            texture,
            tonemap_uniforms,
            tonemap_bind_group,
            width,
            height,
            format,
            mode,
        }
    }

    /// Resize the HDR texture
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let new_texture = texture::Texture::new(
            device,
            width,
            height,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            Some("Hdr::texture"),
        );

        let bind_group = bindgroup::BindGroupBuilder::new()
            .add_texture(
                0,
                new_texture.view.clone(),
                wgpu::ShaderStages::FRAGMENT,
                wgpu::TextureSampleType::Float { filterable: true },
            )
            .add_sampler(
                1,
                new_texture.sampler.clone(),
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::FRAGMENT,
            )
            .build(device, Some("HDR layout"));

        self.texture = new_texture;
        self.bind_group = bind_group;
        self.width = width;
        self.height = height;
    }

    /// Update tonemapping parameters
    pub fn update_tonemap_params(
        &self,
        queue: &wgpu::Queue,
        mode: TonemappingMode,
        peak_brightness_nits: f32,
    ) {
        let uniforms = TonemapUniforms {
            peak_brightness_nits,
            mode: if mode == TonemappingMode::Hdr { 1 } else { 0 },
            _padding: [0.0; 2],
        };
        queue.write_buffer(&self.tonemap_uniforms, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Exposes the HDR texture
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// The format of the HDR texture
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// This renders the internal HDR texture to the [TextureView]
    /// supplied as parameter.
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hdr::process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_bind_group(0, &self.bind_group.bind_group, &[]);
        pass.set_bind_group(1, &self.tonemap_bind_group.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

pub struct HdrLoader {
    texture_format: wgpu::TextureFormat,
    equirect_layout: wgpu::BindGroupLayout,
    equirect_to_cubemap: wgpu::ComputePipeline,
}

impl HdrLoader {
    pub fn new(device: &wgpu::Device) -> Self {
        let module =
            device.create_shader_module(wgpu::include_wgsl!("./shaders/equirectangular.wgsl"));
        let texture_format = wgpu::TextureFormat::Rgba32Float;
        let equirect_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("HdrLoader::equirect_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: texture_format,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&equirect_layout],
            push_constant_ranges: &[],
        });

        let equirect_to_cubemap =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("equirect_to_cubemap"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("compute_equirect_to_cubemap"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            equirect_to_cubemap,
            texture_format,
            equirect_layout,
        }
    }

    pub fn from_equirectangular_bytes(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        dst_size: u32,
        label: Option<&str>,
    ) -> anyhow::Result<texture::CubeTexture> {
        let hdr_decoder = HdrDecoder::new(Cursor::new(data))?;
        let meta = hdr_decoder.metadata();

        // 1. Decode into a DynamicImage
        // HdrDecoder implements ImageDecoder, so we can use DynamicImage::from_decoder
        let dynamic_img = DynamicImage::from_decoder(hdr_decoder)?;

        // 2. Convert to RGBA f32 (The format Rgba32Float expects)
        // .to_rgba32f() returns an ImageBuffer<Rgba<f32>, Vec<f32>>
        let rgba_img = dynamic_img.to_rgba32f();
        let pixels: &[f32] = rgba_img.as_raw();

        // let mut pixels: Vec<u8> = vec![0; hdr_decoder.total_bytes() as usize];
        // hdr_decoder.read_image(&mut pixels)?;

        let src = texture::Texture::new(
            device,
            meta.width,
            meta.height,
            self.texture_format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            None,
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(src.texture.size().width * size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src.texture.size().height),
            },
            src.texture.size(),
        );

        let dst = texture::CubeTexture::create_2d(
            device,
            dst_size,
            dst_size,
            self.texture_format,
            1,
            // We are going to write to `dst` texture so we
            // need to use a `STORAGE_BINDING`.
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Nearest,
            label,
        );

        let dst_view = dst.texture().create_view(&wgpu::TextureViewDescriptor {
            label,
            // Normally, you'd use `TextureViewDimension::Cube`
            // for a cube texture, but we can't use that
            // view dimension with a `STORAGE_BINDING`.
            // We need to access the cube texture layers
            // directly.
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &self.equirect_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        let num_workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&self.equirect_to_cubemap);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

        drop(pass);

        queue.submit([encoder.finish()]);

        Ok(dst)
    }
}
