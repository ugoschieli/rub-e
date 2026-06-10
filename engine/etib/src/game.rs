use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::error::ExternalError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use winit::{application::ApplicationHandler, error::EventLoopError, event_loop::EventLoop};
use winit::window::Fullscreen::Borderless;
use crate::config::EngineConfig;
use crate::input::InputState;
use crate::time::TimeState;
use crate::{Gfx, Scene};

/// All engine-owned state passed to the game on every callback.
///
/// `EngineContext` is the single handle the game receives for graphics,
/// timing, input, configuration, and the egui integration. The window
/// is intentionally hidden — use the provided helper methods instead of
/// accessing it directly.
pub struct EngineContext {
    /// The graphics context (device, queue, surface, depth buffer, …)
    pub gfx: Gfx,
    /// Frame timing and FPS tracking
    pub time: TimeState,
    /// Keyboard input state for the current frame
    pub input: InputState,
    /// Gamepad input state tracker
    pub gilrs: gilrs::Gilrs,
    /// Engine configuration (vsync, HDR, …)
    pub config: Arc<EngineConfig>,
    // window is NOT exposed directly — engine handles it
    window: Arc<Window>,

    /// The egui context used to build UI each frame
    pub egui_ctx: egui::Context,
    /// Winit–egui bridge that converts window events to egui input
    pub egui_state: egui_winit::State,
    /// wgpu renderer for egui primitives
    pub egui_renderer: egui_wgpu::Renderer,
    /// Egui output produced during the current frame's UI pass, consumed by [`EngineContext::render_ui`]
    pub egui_output: Option<egui::FullOutput>,
}

impl EngineContext {
    /// Returns the current physical size of the window in pixels.
    pub fn window_size(&self) -> PhysicalSize<u32> {
        self.window.inner_size()
    }

    /// Set the cursor grab mode (e.g. lock the cursor inside the window).
    pub fn set_cursor_grab(&self, mode: CursorGrabMode) -> Result<(), ExternalError> {
        self.window.set_cursor_grab(mode)
    }

    /// Show or hide the OS cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    /// Set the window title shown in the title bar.
    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Render the egui UI produced during this frame's [`Game::ui`] callback into `view`.
    fn render_ui(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
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
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

    /// Get the current fps
    pub fn fps(&self) -> f32 {
        self.time.fps()
    }

    /// Get the current frame delta
    pub fn dt(&self) -> f32 {
        self.time.dt
    }

    /// Get the current fps
    pub fn now(&self) -> f32 {
        TimeState::now()
    }
}

/// The primary Trait provided by the library your primary game state struct must implement this
pub trait Game {
    /// Parameters forwarded from [`run`] to [`Game::init`].
    type InitParams;

    /// Called once after window + GPU are ready.
    /// `ctx` is fully initialized — no Options, no guards.
    fn init(ctx: &mut EngineContext, params: Self::InitParams) -> Self;

    /// Called every frame before rendering. Update game logic here.
    fn update(&mut self, ctx: &mut EngineContext);
    /// Build egui UI for this frame. Called between `update` and `render`.
    fn ui(&mut self, _ctx: &mut EngineContext, _ui_ctx: &egui::Context) {}
    /// Return the active scene for this frame. The engine renders it automatically after `update`
    /// and resizes it when the window resizes. Switch scenes by returning a different scene.
    fn scene(&mut self) -> Option<&mut Scene> {
        None
    }
    /// Called for window-level input events (keyboard, mouse buttons, …) not consumed by egui.
    fn input(&mut self, _ctx: &mut EngineContext, _event: &WindowEvent) {}
    /// Called for raw device events (e.g. mouse motion deltas) regardless of egui focus.
    fn device_input(&mut self, _ctx: &mut EngineContext, _event: &DeviceEvent) {}
}

struct EngineRunner<G: Game> {
    window: Option<Arc<Window>>,
    ctx: Option<EngineContext>,
    game: Option<G>,
    config: Arc<EngineConfig>,
    params: Option<G::InitParams>,
    /// Raw pointer of the scene rendered last frame — used to detect scene switches.
    last_scene_ptr: Option<*const Scene>,
    /// Last known window size — used to resize a freshly activated scene.
    last_window_size: PhysicalSize<u32>,
}

impl<G: Game> ApplicationHandler for EngineRunner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_fullscreen(Some(Borderless(None))))
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

        let gilrs = gilrs::Gilrs::new().expect("Failed to initialize gilrs");

        let egui_renderer = egui_wgpu::Renderer::new(
            gfx.device(),
            gfx.surface_config.format,
            egui_wgpu::RendererOptions::default(),
        );
        let mut ctx = EngineContext {
            gfx,
            time: TimeState::new(),
            input: InputState::default(),
            gilrs,
            config: self.config.clone(),
            window: window.clone(),
            egui_ctx,
            egui_state,
            egui_renderer,
            egui_output: None,
        };
        let params = self.params.take().unwrap();
        let game = G::init(&mut ctx, params);
        self.last_window_size = window.inner_size();
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
                ctx.gfx.reconfigure_surface_size(size);
                if let Some(scene) = game.scene() {
                    scene.camera.aspect = size.width as f32 / size.height as f32;
                    scene.camera.update_matrix(&ctx.gfx.queue);
                    scene.resize(ctx.gfx.device(), size.width, size.height);
                }
                self.last_window_size = size;
            }
            WindowEvent::RedrawRequested => {
                let (frame, view) = ctx.gfx.get_next_frame();

                // Pump gamepad events
                while let Some(_) = ctx.gilrs.next_event() {}

                let mut scene_encoder =
                    ctx.gfx
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Scene command encoder"),
                        });

                game.update(ctx);
                if let Some(scene) = game.scene() {
                    let ptr = scene as *const Scene;
                    if self.last_scene_ptr != Some(ptr) {
                        let size = self.last_window_size;
                        scene.camera.aspect = size.width as f32 / size.height as f32;
                        scene.camera.update_matrix(&ctx.gfx.queue);
                        scene.resize(ctx.gfx.device(), size.width, size.height);
                        self.last_scene_ptr = Some(ptr);
                    }
                    scene.render(ctx, &mut scene_encoder, &view);
                }
                ctx.gfx.queue.submit(Some(scene_encoder.finish()));

                let raw_input = ctx.egui_state.take_egui_input(&ctx.window);
                let egui_ctx = ctx.egui_ctx.clone();
                let full_output = egui_ctx.run(raw_input, |ui_ctx| {
                    game.ui(ctx, ui_ctx);
                });

                let mut ui_encoder =
                    ctx.gfx
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("UI command encoder"),
                        });

                ctx.egui_state
                    .handle_platform_output(&ctx.window, full_output.platform_output.clone());
                ctx.egui_output = Some(full_output);

                ctx.render_ui(&mut ui_encoder, &view);
                ctx.gfx.queue.submit(Some(ui_encoder.finish()));

                frame.present();

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
        last_scene_ptr: None,
        last_window_size: PhysicalSize::new(0, 0),
    };
    event_loop.run_app(&mut runner)
}
