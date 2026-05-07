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

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
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

    /// Get the current time in seconds since EPOCH
    pub fn now() -> f32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f32()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_state_new() {
        let state = TimeState::new();
        assert_eq!(state.frame_count, 0);
        assert_eq!(state.current_fps, 0.0);
        assert_eq!(state.dt, 0.0);
    }

    #[test]
    fn test_time_state_now() {
        let now1 = TimeState::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let now2 = TimeState::now();
        assert!(now2 >= now1);
        assert!(now2 > 0.0);
    }

    #[test]
    fn test_time_state_tick() {
        let mut state = TimeState::new();
        let initial_last_frame = state.last_frame_time;

        std::thread::sleep(std::time::Duration::from_millis(10));
        state.tick();

        assert!(state.dt > 0.0);
        assert_eq!(state.frame_count, 1);
        assert!(state.last_frame_time > initial_last_frame);
    }

    #[test]
    fn test_time_state_default_and_fps() {
        let state = TimeState::default();
        assert_eq!(state.fps(), 0.0);
    }

    #[test]
    fn test_time_state_fps_update() {
        let mut state = TimeState::new();
        // Force fps_update_time to be > 0.5s ago
        state.fps_update_time = std::time::Instant::now() - std::time::Duration::from_millis(600);
        state.frame_count = 30;
        
        state.tick();
        
        // FPS should be updated now (approx 30 / 0.6 = 50)
        assert!(state.fps() > 0.0);
        assert_eq!(state.frame_count, 0); // Frame count should reset
    }
}
