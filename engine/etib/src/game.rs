use crate::Gfx;
use crate::config::EngineConfig;
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
    pub config: Arc<EngineConfig>,
    // window is NOT exposed directly — engine handles it
    window: Arc<Window>,
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
}

/// The primary Trait provided by the library your primary game state struct must implement this
pub trait Game {
    type InitParams;

    /// Called once after window + GPU are ready.
    /// `ctx` is fully initialized — no Options, no guards.
    fn init(ctx: &mut EngineContext, params: Self::InitParams) -> Self;

    fn update(&mut self, ctx: &mut EngineContext);
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
        let gfx = Gfx::new(window.clone(), &self.config);
        let mut ctx = EngineContext {
            gfx,
            time: TimeState::new(),
            config: self.config.clone(),
            window: window.clone(),
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
        match event {
            WindowEvent::Resized(size) => {
                /* engine handles surface + depth resize, then */
                game.resize(ctx, size);
            }
            WindowEvent::RedrawRequested => {
                game.update(ctx);
                game.render(ctx);
                ctx.time.tick();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            other => game.input(ctx, &other),
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
