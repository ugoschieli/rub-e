use crate::core::surface::Frame;
use crate::game::Game;
use crate::gfx::Gfx;
use winit::window::Window;

pub(crate) struct UiState {
    /// The egui context used to build UI each frame
    pub ctx: egui::Context,
    /// egui-winit bridge that converts window events to egui input
    pub state: egui_winit::State,
    /// wgpu renderer for egui primitives
    pub renderer: egui_wgpu::Renderer,
}

impl UiState {
    pub fn new(window: &Window, gfx: &Gfx) -> Self {
        let ctx = egui::Context::default();
        let viewport_id = egui::ViewportId::ROOT;

        let state = egui_winit::State::new(
            ctx.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = egui_wgpu::Renderer::new(
            &gfx.device,
            gfx.surface.config.format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            ctx,
            state,
            renderer,
        }
    }

    pub fn update<G: Game>(&mut self, game: &mut G, window: &Window) -> egui::FullOutput {
        let raw_input = self.state.take_egui_input(window);
        self.ctx.run_ui(raw_input, |ui| game.ui(ui))
    }

    pub fn render(
        &mut self,
        window: &Window,
        gfx: &Gfx,
        full_output: egui::FullOutput,
        frame: &mut Frame,
    ) {
        let device = &gfx.device;
        let queue = &gfx.queue;

        let primitives = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [gfx.surface.config.width, gfx.surface.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer.update_buffers(
            device,
            queue,
            &mut frame.encoder,
            &primitives,
            &screen_descriptor,
        );

        {
            let render_pass = frame
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui_render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            self.renderer.render(
                &mut render_pass.forget_lifetime(),
                &primitives,
                &screen_descriptor,
            );
        }

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
