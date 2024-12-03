use std::collections::HashSet;

use devotee_backend::Input;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::util::vector::Vector;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

/// Keyboard-related input system.
#[derive(Clone, Default, Debug)]
pub struct Keyboard {
    pressed: HashSet<KeyCode>,
    was_pressed: HashSet<KeyCode>,
}

impl Keyboard {
    /// Create new Keyboard input system instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the key is pressed.
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Check if the key was pressed during the previous tick and not before.
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key) && !self.was_pressed.contains(&key)
    }

    /// Check if the key was released during the previous tick.
    pub fn just_released(&self, key: KeyCode) -> bool {
        !self.pressed.contains(&key) && self.was_pressed.contains(&key)
    }

    /// Check if any key is pressed.
    pub fn is_pressed_any(&self) -> bool {
        !self.pressed.is_empty()
    }

    /// Check if any key was pressed during the previous tick.
    pub fn just_pressed_any(&self) -> bool {
        self.pressed.difference(&self.was_pressed).any(|_| true)
    }
}

impl<EventContext> Input<'_, EventContext> for Keyboard {
    type Event = WindowEvent;

    fn handle_event(&mut self, event: Self::Event, _context: &EventContext) -> Option<Self::Event> {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if let PhysicalKey::Code(code) = event.physical_key {
                match event.state {
                    ElementState::Pressed => self.pressed.insert(code),
                    ElementState::Released => self.pressed.remove(&code),
                };
            }
            None
        } else {
            Some(event)
        }
    }

    fn tick(&mut self) {
        self.was_pressed.clone_from(&self.pressed)
    }
}

/// Mouse position representation.
#[derive(Clone, Copy, Debug)]
pub enum MousePosition {
    /// The mouse is inside the render surface.
    Inside(Vector<i32>),

    /// The mouse is outside the render surface.
    Outside(Vector<i32>),
}

impl MousePosition {
    /// Get the mouse position regardless the mouse being inside or outside the render surface.
    pub fn any(self) -> Vector<i32> {
        match self {
            MousePosition::Inside(inside) => inside,
            MousePosition::Outside(outside) => outside,
        }
    }
}

/// Mouse-related input system.
#[derive(Clone, Debug)]
pub struct Mouse {
    position: MousePosition,
    pressed: HashSet<MouseButton>,
    was_pressed: HashSet<MouseButton>,
}

impl Mouse {
    /// Create new Mouse input system instance.
    pub fn new() -> Self {
        let position = MousePosition::Inside((0, 0).into());
        let pressed = Default::default();
        let was_pressed = Default::default();
        Self {
            position,
            pressed,
            was_pressed,
        }
    }

    /// Check if the button is pressed.
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    /// Check if the button was pressed during the previous tick and not before.
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button) && !self.was_pressed.contains(&button)
    }

    /// Check if the button was released during the previous tick.
    pub fn just_released(&self, button: MouseButton) -> bool {
        !self.pressed.contains(&button) && self.was_pressed.contains(&button)
    }

    /// Get mouse position.
    pub fn position(&self) -> MousePosition {
        self.position
    }
}

impl<EventContext> Input<'_, EventContext> for Mouse
where
    EventContext: backend::EventContext,
{
    type Event = WindowEvent;

    fn handle_event(&mut self, event: Self::Event, context: &EventContext) -> Option<Self::Event> {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => self.pressed.insert(button),
                    ElementState::Released => self.pressed.remove(&button),
                };
                None
            }
            WindowEvent::CursorMoved { position, .. } => {
                match context
                    .position_into_render_surface_space((position.x as f32, position.y as f32))
                {
                    Ok(inside) => {
                        self.position = MousePosition::Inside(inside.into());
                    }
                    Err(outside) => {
                        self.position = MousePosition::Outside(outside.into());
                    }
                }
                None
            }
            _ => Some(event),
        }
    }

    fn tick(&mut self) {
        self.was_pressed.clone_from(&self.pressed)
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard and mouse input systems union.
#[derive(Clone, Debug, Default)]
pub struct KeyboardMouse {
    keyboard: Keyboard,
    mouse: Mouse,
}

impl KeyboardMouse {
    /// Create new input system instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get keyboard subsystem instance reference.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Get mouse subsystem instance reference.
    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
}

impl<EventContext> Input<'_, EventContext> for KeyboardMouse
where
    EventContext: backend::EventContext,
{
    type Event = WindowEvent;

    fn handle_event(&mut self, event: Self::Event, context: &EventContext) -> Option<Self::Event> {
        let event = self.keyboard.handle_event(event, context)?;
        self.mouse.handle_event(event, context)
    }

    fn tick(&mut self) {
        Input::<'_, EventContext>::tick(&mut self.keyboard);
        Input::<'_, EventContext>::tick(&mut self.mouse);
    }
}

/// Cheap input system handling no input event.
#[derive(Debug, Clone, Copy)]
pub struct NoInput;

impl<EventContext> Input<'_, EventContext> for NoInput
where
    EventContext: backend::EventContext,
{
    type Event = WindowEvent;

    fn handle_event(&mut self, event: Self::Event, _: &EventContext) -> Option<Self::Event> {
        Some(event)
    }

    fn tick(&mut self) {}
}
