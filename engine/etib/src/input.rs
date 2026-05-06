use std::collections::HashSet;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Centralized manager for tracking keyboard input state across frames.
#[derive(Default)]
pub struct InputState {
    pressed: HashSet<PhysicalKey>,
    just_pressed: HashSet<PhysicalKey>,
    just_released: HashSet<PhysicalKey>,
}

impl InputState {
    /// Returns true if the key corresponding to `KeyCode` is currently held down.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&PhysicalKey::Code(key))
    }

    /// Returns true if the key corresponding to `KeyCode` was pressed in this exact frame.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&PhysicalKey::Code(key))
    }

    /// Returns true if the key corresponding to `KeyCode` was released in this exact frame.
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&PhysicalKey::Code(key))
    }

    #[cfg(not(tarpaulin_include))]
    pub(crate) fn process_keyboard_event(&mut self, event: &winit::event::KeyEvent) {
        self.process_key_event(event.state, event.physical_key);
    }

    pub(crate) fn process_key_event(&mut self, state: winit::event::ElementState, physical_key: PhysicalKey) {
        let is_pressed = state == winit::event::ElementState::Pressed;

        if is_pressed {
            // insert returns true if the value was not already present
            if self.pressed.insert(physical_key) {
                self.just_pressed.insert(physical_key);
            }
        } else {
            self.pressed.remove(&physical_key);
            self.just_released.insert(physical_key);
        }
    }

    /// Called at the end of the frame by the engine runner to reset transient state.
    pub(crate) fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::ElementState;

    /// process_keyboard_event is a 2-line delegation wrapper:
    ///   fn process_keyboard_event(&mut self, event: &winit::event::KeyEvent) {
    ///       self.process_key_event(event.state, event.physical_key);
    ///   }
    /// winit::event::KeyEvent cannot be constructed in tests (private platform_specific
    /// field), so we verify the delegation by calling process_key_event directly with
    /// the same arguments that process_keyboard_event would pass.
    #[test]
    fn test_process_keyboard_event_delegation() {
        // Simulate the path process_keyboard_event would take when called with a
        // "KeyW Pressed" event: it calls process_key_event(Pressed, Code(KeyW)).
        let mut state = InputState::default();
        let code = KeyCode::KeyW;
        let physical = PhysicalKey::Code(code);

        state.process_key_event(ElementState::Pressed, physical);
        assert!(state.is_key_pressed(code));
        assert!(state.is_key_just_pressed(code));

        state.clear_frame_state();

        state.process_key_event(ElementState::Released, physical);
        assert!(!state.is_key_pressed(code));
        assert!(state.is_key_just_released(code));
    }

    #[test]
    fn test_input_state() {
        let mut state = InputState::default();
        let key = KeyCode::KeyA;
        let physical = PhysicalKey::Code(key);

        assert!(!state.is_key_pressed(key));
        assert!(!state.is_key_just_pressed(key));

        // Press A
        state.process_key_event(ElementState::Pressed, physical);
        assert!(state.is_key_pressed(key));
        assert!(state.is_key_just_pressed(key));
        assert!(!state.is_key_just_released(key));

        // Press A again (auto-repeat)
        state.clear_frame_state();
        state.process_key_event(ElementState::Pressed, physical);
        assert!(state.is_key_pressed(key));
        assert!(!state.is_key_just_pressed(key)); // Not JUST pressed
        assert!(!state.is_key_just_released(key));

        // Release A
        state.clear_frame_state();
        state.process_key_event(ElementState::Released, physical);
        assert!(!state.is_key_pressed(key));
        assert!(!state.is_key_just_pressed(key));
        assert!(state.is_key_just_released(key));

        // Next frame
        state.clear_frame_state();
        assert!(!state.is_key_pressed(key));
        assert!(!state.is_key_just_released(key));
    }

    #[test]
    fn test_multiple_keys_simultaneous() {
        let mut state = InputState::default();
        let a = PhysicalKey::Code(KeyCode::KeyA);
        let b = PhysicalKey::Code(KeyCode::KeyB);

        state.process_key_event(ElementState::Pressed, a);
        state.process_key_event(ElementState::Pressed, b);

        assert!(state.is_key_pressed(KeyCode::KeyA));
        assert!(state.is_key_pressed(KeyCode::KeyB));
        assert!(state.is_key_just_pressed(KeyCode::KeyA));
        assert!(state.is_key_just_pressed(KeyCode::KeyB));

        // Release A, B stays pressed
        state.clear_frame_state();
        state.process_key_event(ElementState::Released, a);
        assert!(!state.is_key_pressed(KeyCode::KeyA));
        assert!(state.is_key_pressed(KeyCode::KeyB));
        assert!(state.is_key_just_released(KeyCode::KeyA));
        assert!(!state.is_key_just_released(KeyCode::KeyB));
    }

    #[test]
    fn test_release_key_never_pressed() {
        let mut state = InputState::default();
        let key = KeyCode::KeyZ;
        let physical = PhysicalKey::Code(key);

        // Releasing a key that was never pressed should not panic and key stays not-pressed
        state.process_key_event(ElementState::Released, physical);
        assert!(!state.is_key_pressed(key));
        assert!(state.is_key_just_released(key)); // it IS in just_released (we "released" it)
    }
}
