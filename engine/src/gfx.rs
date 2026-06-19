use crate::app::App;
use crate::constants::FRAMES_IN_FLIGHT;
use crate::time::Time;
use crate::utils;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

#[derive(Debug)]
pub struct Gfx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface_texture: Option<wgpu::SurfaceTexture>,
    pub surface_texture_view: Option<wgpu::TextureView>,
    pub depth_texture_view: wgpu::TextureView,
    pub encoder: Option<wgpu::CommandEncoder>,
    submission_indices: [Option<wgpu::SubmissionIndex>; FRAMES_IN_FLIGHT],
    pub frame_index: usize,
}

impl Gfx {
    pub fn new(event_loop: &ActiveEventLoop, window: Arc<Window>, app: &mut App) -> Self {
        let size = window.inner_size();

        let instance = utils::create_instance(event_loop);
        let adapter = utils::create_adapter(&instance);
        let (device, queue) = utils::create_device(&adapter);
        let (surface, surface_config) =
            utils::create_surface(&instance, &adapter, &device, window, size);

        let (_depth_texture, depth_texture_view) = utils::create_depth_texture(&device, size);

        Self {
            device,
            queue,
            surface,
            surface_config,
            surface_texture: None,
            surface_texture_view: None,
            depth_texture_view,
            encoder: None,
            submission_indices: [const { None }; FRAMES_IN_FLIGHT],
            frame_index: 0,
        }
    }

    /// Must be called before any `Renderer::render()`
    pub fn update(&mut self, time: &Time) {
        let frame_index = time.frame_number % FRAMES_IN_FLIGHT;
        self.frame_index = frame_index;

        if let Some(idx) = self.submission_indices[frame_index].take() {
            self.device
                .poll(wgpu::PollType::Wait {
                    submission_index: Some(idx),
                    timeout: None,
                })
                .unwrap();
        }

        let Some((current_surface_texture, current_surface_texture_view)) =
            utils::get_current_surface_texture(&self.surface)
        else {
            self.surface_texture = None;
            self.surface_texture_view = None;
            return;
        };
        self.surface_texture = Some(current_surface_texture);
        self.surface_texture_view = Some(current_surface_texture_view);

        self.encoder = Some(utils::create_encoder(&self.device));
    }

    /// Must be called after all `Renderer::render()`
    pub fn submit(&mut self) {
        let encoder = self.encoder.take().unwrap();
        let submission_index = self.queue.submit(Some(encoder.finish()));
        self.submission_indices[self.frame_index] = Some(submission_index);
        self.surface_texture.take().unwrap().present();
    }
}
