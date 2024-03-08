use std::collections::HashSet;

use devotee_backend::Input;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::util::vector::Vector;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

#[derive(Clone, Default, Debug)]
pub struct Keyboard {
    pressed: HashSet<KeyCode>,
    was_pressed: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key) && !self.was_pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        !self.pressed.contains(&key) && self.was_pressed.contains(&key)
    }
}

impl<'a, EventContext> Input<'a, EventContext> for Keyboard {
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
        self.was_pressed = self.pressed.clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MousePosition {
    Inside(Vector<i32>),
    Outside(Vector<i32>),
}

impl MousePosition {
    pub fn any(self) -> Vector<i32> {
        match self {
            MousePosition::Inside(inside) => inside,
            MousePosition::Outside(outside) => outside,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mouse {
    position: MousePosition,
    pressed: HashSet<MouseButton>,
    was_pressed: HashSet<MouseButton>,
}

impl Mouse {
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

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button) && !self.was_pressed.contains(&button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        !self.pressed.contains(&button) && self.was_pressed.contains(&button)
    }

    pub fn position(&self) -> MousePosition {
        self.position
    }
}

impl<'a, EventContext> Input<'a, EventContext> for Mouse
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
        self.was_pressed = self.pressed.clone();
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeyboardMouse {
    keyboard: Keyboard,
    mouse: Mouse,
}

impl KeyboardMouse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
}

impl<'a, EventContext> Input<'a, EventContext> for KeyboardMouse
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
