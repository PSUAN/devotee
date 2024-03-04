use std::collections::HashSet;

use devotee_backend::Input;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::PhysicalKey;

use crate::util::vector::Vector;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

#[derive(Default, Debug)]
pub struct Keyboard {
    pressed: HashSet<KeyCode>,
    was_pressed: HashSet<KeyCode>,
}

impl<'a, EventContext> Input<'a, EventContext> for Keyboard {
    type Event = WindowEvent;

    fn handle_event(&mut self, event: Self::Event, _context: EventContext) -> Option<Self::Event> {
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

    pub fn position(&self) -> MousePosition {
        self.position
    }
}

impl<'a, EventContext> Input<'a, EventContext> for Mouse
where
    EventContext: backend::EventContext,
{
    type Event = WindowEvent;

    fn handle_event(
        &mut self,
        event: Self::Event,
        event_context: EventContext,
    ) -> Option<Self::Event> {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => self.pressed.insert(button),
                    ElementState::Released => self.pressed.remove(&button),
                };
                None
            }
            WindowEvent::CursorMoved { position, .. } => {
                match event_context
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
