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

    /// Helper invoked by the engine to process Winit keyboard events.
    pub(crate) fn process_keyboard_event(&mut self, event: &winit::event::KeyEvent) {
        let is_pressed = event.state == winit::event::ElementState::Pressed;

        if is_pressed {
            // insert returns true if the value was not already present
            if self.pressed.insert(event.physical_key) {
                self.just_pressed.insert(event.physical_key);
            }
        } else {
            self.pressed.remove(&event.physical_key);
            self.just_released.insert(event.physical_key);
        }
    }

    /// Called at the end of the frame by the engine runner to reset transient state.
    pub(crate) fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}
