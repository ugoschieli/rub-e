use std::time::Instant;
use winit::{application::ApplicationHandler, error::EventLoopError, event_loop::EventLoop};

use crate::Gfx;

/// The primary Trait provided by the library your primary game state struct must implement this
pub trait Game: ApplicationHandler {
    /// The function called at the initialization of the engine
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop);

    /// The function called each frame by the engine
    fn render(&mut self);

    /// Getter for a mutable etib::Gfx struct
    fn gfx(&mut self) -> &mut Gfx;

    /// Must return is the graphics state is initialized
    fn is_initialized(&self) -> bool;

    /// Getter for a etib::TimeState struct
    fn time_state(&self) -> &TimeState;
}

/// Launch the game
pub fn run<T: Game>(game: &mut T, event_loop: EventLoop<()>) -> Result<(), EventLoopError> {
    event_loop.run_app(game)
}

/// The struct managing time synchronization
pub struct TimeState {
    /// The time elapsed since the last frame
    pub dt: f32,
    last_time: Instant,
}

impl TimeState {
    /// Calculate the new frame time. MUST BE CALLED EACH FRAME
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_time).as_secs_f32();
        self.dt = dt;
        self.last_time = now;
        log::debug!("dt = {}, fps = {}", 1000. * dt, 1. / dt);
    }
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            dt: 0.,
            last_time: Instant::now(),
        }
    }
}
