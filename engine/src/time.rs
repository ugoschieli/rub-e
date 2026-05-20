use std::time::Instant;

#[derive(Debug, Copy, Clone)]
pub struct Time {
    pub dt: f32,
    pub now: Instant,
    last_instant: Instant,
    pub frame_number: usize,
}

impl Time {
    pub fn new() -> Self {
        Self {
            dt: 0.,
            now: Instant::now(),
            last_instant: Instant::now(),
            frame_number: 0,
        }
    }

    pub fn tick(&mut self) {
        self.now = Instant::now();
        self.dt = (self.now - self.last_instant).as_secs_f32();
        self.last_instant = self.now;
        self.frame_number += 1;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
