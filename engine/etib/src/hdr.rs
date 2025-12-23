use wgpu::Operations;

use etib_core::bindgroup;
use etib_core::pipeline;
use etib_core::shader;
use etib_core::texture;

/// Owns the render texture and controls tonemapping
pub struct HdrPipeline {
    pipeline: pipeline::Pipeline,
    bind_group: bindgroup::BindGroup,
    texture: texture::Texture,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl HdrPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
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

        let shader_str = include_str!("./shaders/hdr.wgsl");
        let shader = shader::Shader::new(shader_str, device, Some("HDR shader"));

        let pipeline = pipeline::Pipeline::new_v2(
            device,
            &[&bind_group.layout],
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
            width,
            height,
            format,
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
        pass.draw(0..3, 0..1);
    }
}
