use std::sync::Arc;

use winit::dpi::PhysicalSize;
use winit::error::ExternalError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::Fullscreen::Borderless;
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use winit::{application::ApplicationHandler, error::EventLoopError, event_loop::EventLoop};

use crate::config::EngineConfig;
use crate::gfx::{Frame, Gfx};
use crate::input::InputState;
use crate::time::Time;
use crate::ui::UiState;

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
    pub time: Time,
    /// Keyboard input state for the current frame
    pub input: InputState,
    /// Gamepad input state tracker
    pub gilrs: gilrs::Gilrs,
    /// Engine configuration (vsync, HDR, …)
    pub config: Arc<EngineConfig>,
    /// window is NOT exposed directly — engine handles it
    window: Arc<Window>,
    ui: UiState,
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
    pub fn set_cursor_visible(&self, visible: bool) {
        self.window.set_cursor_visible(visible);
    }

    /// Set the window title shown in the title bar.
    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
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

    /// Render to the current frame with access to the `CommandEncoder` inside `Frame`
    fn render(&mut self, ctx: &mut EngineContext, frame: &mut Frame);

    /// Build egui UI for this frame. Called between `update` and `render`.
    fn ui(&mut self, ui: &mut egui::Ui) {}

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

        let gfx = Gfx::new(window.clone(), &self.config);

        let ui = UiState::new(&window, &gfx);

        let gilrs = gilrs::Gilrs::new().expect("Failed to initialize gilrs");

        let mut ctx = EngineContext {
            gfx,
            time: Time::new(),
            input: InputState::default(),
            gilrs,
            config: self.config.clone(),
            window: window.clone(),
            ui,
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

        let response = ctx.ui.state.on_window_event(&ctx.window, &event);
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
                self.last_window_size = size;
            }
            WindowEvent::RedrawRequested => {
                game.update(ctx);
                // Pump gamepad events
                while let Some(_) = ctx.gilrs.next_event() {}

                let frame = ctx.gfx.begin_frame();
                if let Some(mut frame) = frame {
                    game.render(ctx, &mut frame);

                    let ui_output = ctx.ui.update(game, &ctx.window);
                    ctx.ui.render(&ctx.window, &ctx.gfx, ui_output, &mut frame);

                    ctx.gfx.end_frame(frame);
                }

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
    config: Option<EngineConfig>,
    params: Option<G::InitParams>,
) -> Result<(), EventLoopError> {
    let config = config.or_else(|| Some(EngineConfig::load_from_file("config.json")));

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut runner = EngineRunner::<G> {
        window: None,
        ctx: None,
        game: None,
        config: Arc::new(config.unwrap()),
        params,
        last_window_size: PhysicalSize::new(0, 0),
    };
    event_loop.run_app(&mut runner)
}
