use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::config::EngineConfig;
use crate::constants::{DEPTH_FORMAT, FRAMES_IN_FLIGHT};
use crate::core::compute::{create_compute_pass, create_compute_pipeline};
use crate::core::surface::{Frame, Surface};
use crate::core::texture::Texture;

#[derive(Debug)]
pub struct Gfx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: Surface,
    pub depth: Texture,
    submission_indices: [Option<wgpu::SubmissionIndex>; FRAMES_IN_FLIGHT],
    pub frame_index: usize,
}

impl Gfx {
    pub fn new(window: Arc<Window>, _config: &EngineConfig) -> Self {
        let size = window.inner_size();

        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);

        let surface = Surface::new(&instance, &adapter, &device, window, size);
        let depth = Texture::new_2d(
            &device,
            "depth_texture",
            DEPTH_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
            size.width,
            size.height,
        );

        Self {
            device,
            queue,
            surface,
            depth,
            submission_indices: [const { None }; FRAMES_IN_FLIGHT],
            frame_index: 0,
        }
    }

    fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env())
    }

    fn create_adapter(instance: &wgpu::Instance) -> wgpu::Adapter {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .expect("Failed to obtain an adapter")
    }

    fn create_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        let adapter_limits = adapter.limits();
        log::info!("{adapter_limits:#?}");

        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::default() | wgpu::Features::BGRA8UNORM_STORAGE,
            required_limits: adapter_limits,
            experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
            ..Default::default()
        }))
        .expect("Failed to obtain a device")
    }

    pub fn begin_frame(&mut self) -> Option<Frame> {
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

        self.surface.get_current_texture(&self.device)
    }

    pub fn end_frame(&mut self, frame: Frame) {
        let submission_index = self.queue.submit(Some(frame.encoder.finish()));
        self.submission_indices[self.frame_index] = Some(submission_index);
        frame.surface_texture.present();
        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT;
    }

    pub fn reconfigure_surface_size(&mut self, size: PhysicalSize<u32>) {
        self.surface.resize(&self.device, size);
        self.depth.resize_2d(&self.device, size.width, size.height);
    }

    pub fn create_texture_2d(
        &self,
        label: &str,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        width: u32,
        height: u32,
    ) -> Texture {
        Texture::new_2d(&self.device, label, format, usage, width, height)
    }

    pub fn create_compute_pipeline(
        &self,
        label: &str,
        bind_group_layout: Option<&wgpu::BindGroupLayout>,
        immediate_size: u32,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::ComputePipeline {
        create_compute_pipeline(
            &self.device,
            label,
            bind_group_layout,
            immediate_size,
            shader,
        )
    }

    pub fn create_compute_pass<'encoder>(
        &self,
        label: &str,
        encoder: &'encoder mut wgpu::CommandEncoder,
    ) -> wgpu::ComputePass<'encoder> {
        create_compute_pass(label, encoder)
    }
}
