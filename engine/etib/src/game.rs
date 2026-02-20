use crate::Gfx;
use crate::config::EngineConfig;
use crate::input::InputState;
use crate::time::TimeState;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::error::ExternalError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use winit::{application::ApplicationHandler, error::EventLoopError, event_loop::EventLoop};

// What the engine provides:
pub struct EngineContext {
    pub gfx: Gfx,
    pub time: TimeState,
    pub input: InputState,
    pub config: Arc<EngineConfig>,
    // window is NOT exposed directly — engine handles it
    window: Arc<Window>,

    // egui integration
    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
    pub egui_output: Option<egui::FullOutput>,
}

impl EngineContext {
    pub fn window_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(mode)
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    pub fn render_ui(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let Some(full_output) = self.egui_output.take() else {
            return;
        };

        let device = self.gfx.device();
        let queue = &self.gfx.queue;

        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [
                self.gfx.surface_config.width,
                self.gfx.surface_config.height,
            ],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.egui_renderer
            .update_buffers(device, queue, encoder, &primitives, &screen_descriptor);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
            });

            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &primitives,
                &screen_descriptor,
            );
        }

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
    }
}

/// The primary Trait provided by the library your primary game state struct must implement this
pub trait Game {
    type InitParams;

    /// Called once after window + GPU are ready.
    /// `ctx` is fully initialized — no Options, no guards.
    fn init(ctx: &mut EngineContext, params: Self::InitParams) -> Self;

    fn update(&mut self, ctx: &mut EngineContext);
    fn ui(&mut self, _ctx: &mut EngineContext, _ui_ctx: &egui::Context) {}
    fn render(&mut self, ctx: &mut EngineContext);
    fn resize(&mut self, ctx: &mut EngineContext, size: PhysicalSize<u32>);
    fn input(&mut self, ctx: &mut EngineContext, event: &WindowEvent) {}
    fn device_input(&mut self, ctx: &mut EngineContext, event: &DeviceEvent) {}
}

struct EngineRunner<G: Game> {
    window: Option<Arc<Window>>,
    ctx: Option<EngineContext>,
    game: Option<G>,
    config: Arc<EngineConfig>,
    params: Option<G::InitParams>,
}

impl<G: Game> ApplicationHandler for EngineRunner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .unwrap(),
        );

        let egui_ctx = egui::Context::default();
        let viewport_id = egui::ViewportId::ROOT;
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let gfx = Gfx::new(window.clone(), &self.config);

        let egui_renderer = egui_wgpu::Renderer::new(
            gfx.device(),
            gfx.surface_config.format,
            egui_wgpu::RendererOptions::default(),
        );
        let mut ctx = EngineContext {
            gfx,
            time: TimeState::new(),
            input: InputState::default(),
            config: self.config.clone(),
            window: window.clone(),
            egui_ctx,
            egui_state,
            egui_renderer,
            egui_output: None,
        };
        let params = self.params.take().unwrap();
        let game = G::init(&mut ctx, params);
        self.window = Some(window);
        self.ctx = Some(ctx);
        self.game = Some(game);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let (Some(ctx), Some(game)) = (&mut self.ctx, &mut self.game) else {
            return;
        };

        let response = ctx.egui_state.on_window_event(&ctx.window, &event);
        if response.repaint {
            ctx.window.request_redraw();
        }

        match event {
            WindowEvent::KeyboardInput {
                event: ref kb_event,
                ..
            } => {
                ctx.input.process_keyboard_event(kb_event);
                if !response.consumed {
                    game.input(ctx, &event);
                }
            }
            WindowEvent::Resized(size) => {
                /* engine handles surface + depth resize, then */
                game.resize(ctx, size);
            }
            WindowEvent::RedrawRequested => {
                let raw_input = ctx.egui_state.take_egui_input(&ctx.window);
                let egui_ctx = ctx.egui_ctx.clone();
                let full_output = egui_ctx.run(raw_input, |ui_ctx| {
                    game.ui(ctx, ui_ctx);
                });

                ctx.egui_state
                    .handle_platform_output(&ctx.window, full_output.platform_output.clone());
                ctx.egui_output = Some(full_output);

                game.update(ctx);
                game.render(ctx);
                ctx.time.tick();
                ctx.input.clear_frame_state();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            other => {
                if !response.consumed {
                    game.input(ctx, &other)
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let (Some(ctx), Some(game)) = (&mut self.ctx, &mut self.game) else {
            return;
        };
        game.device_input(ctx, &event);
    }
}

/// Launch the game
pub fn run<G: Game>(
    config: EngineConfig,
    params: Option<G::InitParams>,
) -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = EngineRunner::<G> {
        window: None,
        ctx: None,
        game: None,
        config: Arc::new(config),
        params,
    };
    event_loop.run_app(&mut runner)
}
