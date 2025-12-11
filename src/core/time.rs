/// The struct managing time synchronization
#[derive(Debug)]
pub struct TimeState {
    /// The last frame time
    pub last_frame_time: std::time::Instant,
    /// The frame count
    pub frame_count: u32,
    /// The time when the last FPS update was performed
    pub fps_update_time: std::time::Instant,
    /// The current FPS
    pub current_fps: f32,
    /// The delta time since the last frame
    pub dt: f32,
}

impl TimeState {
    /// Create a new TimeState
    pub fn new() -> Self {
        Self {
            last_frame_time: std::time::Instant::now(),
            frame_count: 0,
            fps_update_time: std::time::Instant::now(),
            current_fps: 0.,
            dt: 0.,
        }
    }

    /// Get the current FPS
    pub fn fps(&self) -> f32 {
        self.current_fps
    }

    /// Calculate the new frame time. MUST BE CALLED EACH FRAME
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.dt = dt;

        self.frame_count += 1;
        let fps_elapsed = now.duration_since(self.fps_update_time).as_secs_f32();
        if fps_elapsed >= 0.5 {
            self.current_fps = self.frame_count as f32 / fps_elapsed;
            self.frame_count = 0;
            self.fps_update_time = now;
        }
    }
}
