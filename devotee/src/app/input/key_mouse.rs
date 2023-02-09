use std::collections::HashSet;

use pixels::Pixels;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, WindowEvent};

use super::Input;
use crate::util::vector::Vector;

pub use winit::event::VirtualKeyCode;

/// The naive keyboard and mouse input handler.
#[derive(Clone, Default)]
pub struct KeyMouse {
    keyboard: Keyboard,
    mouse: Mouse,
}

impl KeyMouse {
    /// Get the `Keyboard` part.
    pub fn keys(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Get the `Mouse` part.
    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
}

/// Keyboard part of the `KeyMouse` input handler.
#[derive(Clone, Default)]
pub struct Keyboard {
    currently_pressed: HashSet<VirtualKeyCode>,
    previously_pressed: HashSet<VirtualKeyCode>,
}

impl Keyboard {
    /// Check if the specified key is currently pressed.
    pub fn is_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key)
    }

    /// Check if the specified key was pressed just before this update call.
    pub fn just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.currently_pressed.contains(&key) & !self.previously_pressed.contains(&key)
    }

    /// Check if the specified key was released just before this update call.
    pub fn just_released(&self, key: VirtualKeyCode) -> bool {
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

/// Mouse part of the `KeyMouse` input handler.
#[derive(Clone, Default)]
pub struct Mouse {
    position: Option<Vector<i32>>,
    currently_pressed: HashSet<MouseButton>,
    previously_pressed: HashSet<MouseButton>,
}

impl Mouse {
    /// Get current mouse position if any is present.
    pub fn position(&self) -> Option<Vector<i32>> {
        self.position
    }

    /// Check if specific mouse button is currently pressed.
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.currently_pressed.contains(&button)
    }

    /// Check if specific mouse button was pressed just before this update call.
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.currently_pressed.contains(&button) & !self.previously_pressed.contains(&button)
    }

    /// Check if specific mouse button was released just before this update call.
    pub fn just_released(&self, button: MouseButton) -> bool {
        !self.currently_pressed.contains(&button) & self.previously_pressed.contains(&button)
    }

    fn register_button_press_event(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => self.currently_pressed.insert(button),
            ElementState::Released => self.currently_pressed.remove(&button),
        };
    }

    fn register_cursor_left(&mut self) {
        self.position = None;
    }

    fn register_cursor_moved(&mut self, position: PhysicalPosition<f64>, pixels: &Pixels) {
        self.position = Some(match pixels.window_pos_to_pixel(position.into()) {
            Ok(in_bounds) => (in_bounds.0 as i32, in_bounds.1 as i32).into(),
            Err(out_of_bounds) => (out_of_bounds.0 as i32, out_of_bounds.1 as i32).into(),
        });
    }
}

impl Input for KeyMouse {
    fn next_frame(&mut self) {
        self.keyboard.step();
    }

    fn consume_window_event<'a>(
        &mut self,
        event: WindowEvent<'a>,
        pixels: &Pixels,
    ) -> Option<WindowEvent<'a>> {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                self.keyboard.register_key_event(input);
                None
            }
            WindowEvent::CursorLeft { .. } => {
                self.mouse.register_cursor_left();
                None
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.register_cursor_moved(position, pixels);
                None
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse.register_button_press_event(button, state);
                None
            }
            event => Some(event),
        }
    }
}
