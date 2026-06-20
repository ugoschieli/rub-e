use crate::constants::FRAMES_IN_FLIGHT;
use wgpu::CurrentSurfaceTexture;
use winit::dpi::PhysicalSize;

#[derive(Debug)]
pub struct Surface {
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface<'static>,
}

#[derive(Debug)]
pub struct Frame {
    pub surface_texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

impl Surface {
    pub fn new(
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        size: PhysicalSize<u32>,
    ) -> Self {
        let surface = instance
            .create_surface(window)
            .expect("Failed to create a surface");

        let mut config = surface
            .get_default_config(adapter, size.width, size.height)
            .expect("The surface isn't supported by this adapter");
        config.desired_maximum_frame_latency = u32::try_from(FRAMES_IN_FLIGHT).unwrap();
        config.present_mode = wgpu::PresentMode::AutoVsync;
        config.present_mode = wgpu::PresentMode::Immediate;
        log::info!("{config:?}");

        surface.configure(device, &config);

        Self { config, surface }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&device, &self.config);
    }

    pub fn get_current_texture(&self, device: &wgpu::Device) -> Option<Frame> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => {
                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                Some(Frame {
                    surface_texture,
                    view,
                    encoder,
                })
            }
            _ => None,
        }
    }
}
