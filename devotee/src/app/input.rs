use std::collections::HashSet;
pub use winit::event::VirtualKeyCode;
use winit::event::{ElementState, KeyboardInput, WindowEvent};

/// Input trait.
/// Specifies input storing.
pub trait Input {
    /// Handle frame change.
    fn next_frame(&mut self);
    /// Register `winit` event.
    /// Return `None` if the event is consumed.
    fn consume_window_event<'a>(&mut self, event: WindowEvent<'a>) -> Option<WindowEvent<'a>>;
}

/// The simple keyboard input handler.
#[derive(Clone, Default)]
pub struct Keyboard {
    currently_pressed: HashSet<VirtualKeyCode>,
    previously_pressed: HashSet<VirtualKeyCode>,
}

impl Keyboard {
    /// Check if the specified key is currently pressed.
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key)
    }

    /// Check if the specified key was just pressed before this update call.
    pub fn just_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key) & !self.previously_pressed.contains(&key)
    }

    /// Check if the specified key was just released before this update call.
    pub fn just_key_released(&self, key: VirtualKeyCode) -> bool {
        !self.currently_pressed.contains(&key) & self.previously_pressed.contains(&key)
    }

    fn step(&mut self) {
        self.previously_pressed = self.currently_pressed.clone();
    }

    fn register_key_event(&mut self, event: KeyboardInput) {
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

impl Input for Keyboard {
    fn next_frame(&mut self) {
        self.step();
    }

    fn consume_window_event<'a>(&mut self, event: WindowEvent<'a>) -> Option<WindowEvent<'a>> {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            self.register_key_event(input);
            None
        } else {
            Some(event)
        }
    }
}
