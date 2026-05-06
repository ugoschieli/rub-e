use std::sync::Arc;

use crate::config::EngineConfig;
use crate::utils::wgpu_utils;
use winit::{dpi::PhysicalSize, window::Window};

/// A wrapper around muliple wgpu structs that holds the graphics state of the app
pub struct Gfx {
    /// The wgpu Instance
    pub instance: wgpu::Instance,
    /// The wgpu Surface
    pub surface: wgpu::Surface<'static>,
    /// The wgpu SurfaceConfiguration
    pub surface_config: wgpu::SurfaceConfiguration,
    /// The wgpu device
    pub device: wgpu::Device,
    /// The wgpu Queue
    pub queue: wgpu::Queue,
    /// The Depth Buffer Texture
    pub depth_texture: wgpu::Texture,
    /// The Depth Buffer Texture View
    pub depth_texture_view: wgpu::TextureView,
    /// Whether HDR rendering is active
    pub is_hdr_active: bool,
    /// HDR peak brightness in nits
    pub peak_brightness_nits: f32,
}

impl Gfx {
    /// Create a new wgpu Instance and initialize the Gfx struct.
    /// Need to be called only once at the initialization of an app.
    ///
    /// # Arguments
    /// * `window` - The window to create the graphics context for
    /// * `config_path` - Optional path to the config file. Defaults to "config.json" if None.
    #[cfg(not(tarpaulin_include))]
    pub fn new(window: Arc<Window>, config: &EngineConfig) -> Gfx {
        let window_size = window.inner_size();

        let instance = wgpu_utils::create_instance();
        let surface = instance.create_surface(window).unwrap();
        let adapter = wgpu_utils::create_adapter(&instance, &surface).unwrap();
        let (device, queue) = pollster::block_on(wgpu_utils::create_device(&adapter)).unwrap();

        let (surface_config, is_hdr_active) =
            wgpu_utils::configure_surface(&adapter, &device, &surface, window_size, &config);

        log::info!(
            "HDR rendering: {}",
            if is_hdr_active { "ACTIVE" } else { "INACTIVE" }
        );

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Gfx {
            instance,
            surface,
            surface_config,
            device,
            queue,
            depth_texture,
            depth_texture_view,
            is_hdr_active,
            peak_brightness_nits: config.peak_brightness_nits,
        }
    }

    /// Getter function to obtain the wgpu::Device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the texture and view of the next frame that will be rendered
    #[cfg(not(tarpaulin_include))]
    pub fn get_next_frame(&self) -> (wgpu::SurfaceTexture, wgpu::TextureView) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        (frame, view)
    }

    /// Get a default wgpu::RenderPassDescriptor with depth testing enabled
    #[cfg(not(tarpaulin_include))]
    pub fn render_pass<'gfx: 'tex, 'tex>(
        &'gfx self,
        color_attachments: &'tex [Option<wgpu::RenderPassColorAttachment<'tex>>],
        label: &'tex str,
    ) -> wgpu::RenderPassDescriptor<'tex> {
        wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0), // Far plane is 1.0 (reversed Z would use 0.0)
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        }
    }

    /// Helper function to create a default wgpu::RenderPassDescriptor from a wgpu::TextureView
    pub fn color_attachments_from_view(
        view: &'_ wgpu::TextureView,
    ) -> wgpu::RenderPassColorAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        }
    }

    /// Helper function to reconfigure the Surface size. Needs to be called when the window is
    /// resized.
    #[cfg(not(tarpaulin_include))]
    pub fn reconfigure_surface_size(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        // Recreate depth buffer with new size
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
        });

        self.depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
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

    /// color_attachments_from_view is a pure function: it wraps the given TextureView
    /// into a RenderPassColorAttachment with Load::Clear(BLACK) and Store.
    #[test]
    fn test_color_attachments_from_view() {
        let Some((device, _)) = make_device() else { return; };

        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: 4, height: 4, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        let attachment = Gfx::color_attachments_from_view(&view);
        assert!(attachment.resolve_target.is_none());
        match attachment.ops.load {
            wgpu::LoadOp::Clear(c) => {
                assert_eq!(c, wgpu::Color::BLACK);
            }
            _ => panic!("expected LoadOp::Clear(BLACK)"),
        }
        assert_eq!(attachment.ops.store, wgpu::StoreOp::Store);
    }
}
