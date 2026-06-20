use std::time::Instant;

/// The struct managing time synchronization
#[derive(Debug)]
pub struct Time {
    /// The last frame time
    pub now: Instant,
    /// The frame count
    pub frame_number: usize,
    /// The delta time since the last frame
    pub dt: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    /// Create a new `TimeState`
    pub fn new() -> Self {
        Self {
            now: Instant::now(),
            frame_number: 0,
            dt: 0.,
        }
    }

    /// Calculate the new frame time. MUST BE CALLED EACH FRAME
    pub fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.now).as_secs_f32();
        self.now = now;
        self.dt = dt;
        self.frame_number += 1;
    }
}
