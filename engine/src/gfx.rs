use crate::config::EngineConfig;
use crate::constants::FRAMES_IN_FLIGHT;
use crate::time::Time;
use crate::utils;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(Debug)]
pub struct Gfx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_texture_view: Option<wgpu::TextureView>,
    pub depth_texture: Option<wgpu::Texture>,
    pub depth_texture_view: wgpu::TextureView,
    pub encoder: Option<wgpu::CommandEncoder>,
    submission_indices: [Option<wgpu::SubmissionIndex>; FRAMES_IN_FLIGHT],
    pub frame_index: usize,
}

impl Gfx {
    pub fn new(window: Arc<Window>, _config: &EngineConfig) -> Self {
        let size = window.inner_size();

        let instance = utils::create_instance();
        let adapter = utils::create_adapter(&instance);
        let (device, queue) = utils::create_device(&adapter);
        let (surface, surface_config) =
            utils::create_surface(&instance, &adapter, &device, window, size);

        let (depth_texture, depth_texture_view) = utils::create_depth_texture(&device, size);

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_texture: None,
            surface_texture_view: None,
            depth_texture: Some(depth_texture),
            depth_texture_view,
            encoder: None,
            submission_indices: [const { None }; FRAMES_IN_FLIGHT],
            frame_index: 0,
        }
    }

    /// Submit the recorded scene commands. Does **not** present — call
    /// [`Gfx::present`] after any UI pass so the whole frame lands in one
    /// swapchain image.
    pub fn submit(&mut self) {
        let encoder = self.encoder.take().unwrap();
        let submission_index = self.queue.submit(Some(encoder.finish()));
        self.submission_indices[self.frame_index] = Some(submission_index);
    }

    /// Present the acquired frame. Call after [`Gfx::submit`] and any UI pass.
    pub fn present(&mut self) {
        self.surface_texture_view = None;
        if let Some(frame) = self.surface_texture.take() {
            frame.present();
        }
    }

    /// Begin a frame: advance the frames-in-flight ring, wait for the GPU work
    /// that previously used this ring slot to finish, then acquire the next
    /// swapchain frame and store its texture + view (and a fresh encoder) in
    /// `self` so renderers and [`Gfx::submit`]/[`Gfx::present`] can reach them.
    ///
    /// Returns `false` when no frame is available (e.g. surface occluded), in
    /// which case the caller should skip rendering this frame.
    pub fn acquire_frame(&mut self, time: &Time) -> bool {
        self.frame_index = time.frame_number % FRAMES_IN_FLIGHT;

        // Block until the submission that last used this ring slot has
        // completed, so we never write buffers the GPU is still reading.
        if let Some(idx) = self.submission_indices[self.frame_index].take() {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(idx),
                    timeout: None,
                })
                .unwrap();
        }

        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => {
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                self.surface_texture = Some(frame);
                self.surface_texture_view = Some(view);
                self.encoder = Some(utils::create_encoder(&self.device));
                true
            }
            _ => false,
        }
    }

    pub fn reconfigure_surface_size(&mut self, size: PhysicalSize<u32>) {
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        // Recreate depth buffer with new size
        self.depth_texture = Some(self.device.create_texture(&wgpu::TextureDescriptor {
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
        }));

        self.depth_texture_view = self
            .depth_texture
            .as_ref()
            .unwrap()
            .create_view(&wgpu::TextureViewDescriptor::default());
    }
}
