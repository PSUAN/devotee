use std::collections::HashSet;
pub use winit::event::VirtualKeyCode;
use winit::event::{ElementState, KeyboardInput};

/// The simple keyboard input handler.
#[derive(Clone, Default)]
pub struct Input {
    currently_pressed: HashSet<VirtualKeyCode>,
    previously_pressed: HashSet<VirtualKeyCode>,
}

impl Input {
    /// Check if the specified key is currently pressed.
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key)
    }

    /// Check if the specified key was pressed just before this update.
    pub fn just_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key) & !self.previously_pressed.contains(&key)
    }

    pub(super) fn step(&mut self) {
        self.previously_pressed = self.currently_pressed.clone();
    }

    pub(super) fn register_key_event(&mut self, event: KeyboardInput) {
        if let Some(keycode) = event.virtual_keycode {
            match event.state {
                ElementState::Pressed => self.register_key_pressed(keycode),
                ElementState::Released => self.register_key_released(keycode),
            }
        }
    }

    fn register_key_pressed(&mut self, key: VirtualKeyCode) {
        self.currently_pressed.insert(key);
    }

    fn register_key_released(&mut self, key: VirtualKeyCode) {
        self.currently_pressed.remove(&key);
    }
}
