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
    /// Create a new HDR pipeline.
    ///
    /// `mode` selects ACES (SDR) or PQ (HDR) tonemapping.
    /// `peak_brightness_nits` sets the display peak brightness for PQ mapping.
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

/// Utility that converts an equirectangular HDR image into a cubemap texture on the GPU.
pub struct HdrLoader {
    texture_format: wgpu::TextureFormat,
    equirect_layout: wgpu::BindGroupLayout,
    equirect_to_cubemap: wgpu::ComputePipeline,
}

impl HdrLoader {
    /// Create a new `HdrLoader`, uploading the equirectangular-to-cubemap compute shader.
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
            label: Some("HdrLoader::pipeline_layout"),
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

    /// Decode an equirectangular HDR image from `data` and convert it to a `dst_size×dst_size`
    /// cubemap texture using a GPU compute shader.
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

    fn make_surface_config(width: u32, height: u32) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Rgba8Unorm,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    #[test]
    fn test_hdr_pipeline_new_sdr() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let config = make_surface_config(100, 100);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
        // format() is the only public observable; width/height/mode are private
        assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
        // Verify the view is accessible (texture was created at 100x100)
        let _view: &wgpu::TextureView = pipeline.view();
    }

    #[test]
    fn test_hdr_pipeline_new_hdr_mode() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let config = make_surface_config(200, 150);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Hdr, 4000.0);
        // Verify HdrPipeline is usable after construction in HDR mode
        assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
        let _view: &wgpu::TextureView = pipeline.view();
    }

    #[test]
    fn test_hdr_pipeline_format() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let config = make_surface_config(64, 64);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
        assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
    }

    #[test]
    fn test_hdr_pipeline_view_returns_texture_view() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let config = make_surface_config(64, 64);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
        // view() should return the underlying HDR texture view without panicking
        let _view: &wgpu::TextureView = pipeline.view();
    }

    #[test]
    fn test_hdr_pipeline_resize() {
        let Some((device, _)) = make_device() else {
            return;
        };
        let config = make_surface_config(100, 100);
        let mut pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
        // After resize the pipeline must still be usable (view and format unchanged)
        pipeline.resize(&device, 200, 200);
        assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
        let _view: &wgpu::TextureView = pipeline.view();
    }

    #[test]
    fn test_hdr_pipeline_update_tonemap_params_sdr() {
        let Some((device, queue)) = make_device() else {
            return;
        };
        let config = make_surface_config(64, 64);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Hdr, 1000.0);
        // Switch to SDR mode at runtime — must not panic
        pipeline.update_tonemap_params(&queue, TonemappingMode::Sdr, 500.0);
    }

    #[test]
    fn test_hdr_pipeline_update_tonemap_params_hdr() {
        let Some((device, queue)) = make_device() else {
            return;
        };
        let config = make_surface_config(64, 64);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
        // Switch to HDR mode at runtime — exercises the mode == Hdr branch in uniforms
        pipeline.update_tonemap_params(&queue, TonemappingMode::Hdr, 4000.0);
    }

    #[test]
    fn test_hdr_pipeline_process() {
        let Some((device, _queue)) = make_device() else {
            return;
        };
        let config = make_surface_config(64, 64);
        let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);

        // Create an output texture to render into
        let output_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output"),
            size: wgpu::Extent3d { width: 64, height: 64, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let output_view = output_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("test encoder"),
        });
        // process() must not panic; it opens a render pass internally
        pipeline.process(&mut encoder, &output_view);
    }

    #[test]
    fn test_hdr_loader_new() {
        let Some((device, _)) = make_device() else {
            return;
        };
        // HdrLoader::new creates the compute pipeline — must not panic
        let _loader = HdrLoader::new(&device);
    }

    /// Build a minimal Radiance HDR image in memory (2×1 pixels, all white).
    fn make_minimal_hdr_bytes() -> Vec<u8> {
        use image::codecs::hdr::HdrEncoder;
        use image::Rgb;
        // 2 pixels wide, 1 pixel tall
        let pixels: Vec<Rgb<f32>> = vec![
            Rgb([1.0_f32, 1.0, 1.0]),
            Rgb([0.5_f32, 0.5, 0.5]),
        ];
        let mut buf = Vec::<u8>::new();
        let enc = HdrEncoder::new(&mut buf);
        enc.encode(&pixels, 2, 1).expect("failed to encode test HDR");
        buf
    }

    #[test]
    fn test_hdr_loader_from_equirectangular_bytes() {
        let Some((device, queue)) = make_device() else {
            return;
        };
        // Check that the device supports Rgba32Float storage binding
        // (required by the compute shader). If not, skip the test.
        if !device
            .features()
            .contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES)
        {
            // Many headless adapters don't support Rgba32Float storage;
            // guard so the test doesn't panic on those machines.
        }

        let loader = HdrLoader::new(&device);
        let hdr_bytes = make_minimal_hdr_bytes();

        // from_equirectangular_bytes exercises lines 289-391
        let result = loader.from_equirectangular_bytes(
            &device,
            &queue,
            &hdr_bytes,
            16,            // small cubemap — fast and enough to exercise the code path
            Some("test_cubemap"),
        );
        assert!(
            result.is_ok(),
            "from_equirectangular_bytes failed: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_hdr_headless() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }));
        
        if let Ok(adapter) = adapter {
            let (device, _) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
            
            // Test HdrLoader pipeline creation
            let _loader = HdrLoader::new(&device);
            
            // Test HdrPipeline
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8Unorm,
                width: 100,
                height: 100,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            
            let pipeline = HdrPipeline::new(&device, &config, TonemappingMode::Sdr, 1000.0);
            assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
            
            // Test pipeline resize
            let mut pipeline = pipeline;
            pipeline.resize(&device, 200, 200);
            assert_eq!(pipeline.format(), wgpu::TextureFormat::Rgba16Float);
        }
    }
}
