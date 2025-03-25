use std::collections::HashSet;

use backend::middling::{self, InputHandler};
use winit::dpi::PhysicalPosition;
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

impl<Context> InputHandler<WindowEvent, Context> for Keyboard {
    fn handle_event(&mut self, event: WindowEvent, _context: &Context) -> Option<WindowEvent> {
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

    fn update(&mut self) {
        self.was_pressed.clone_from(&self.pressed)
    }
}

/// Mouse position representation.
pub type MousePosition = Option<Vector<i32>>;

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
        let position = None;
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

impl<EventContext, SurfaceSpace> InputHandler<WindowEvent, EventContext> for Mouse
where
    EventContext: middling::EventContext<
        PhysicalPosition<f64>,
        SurfaceSpace = Option<PhysicalPosition<SurfaceSpace>>,
    >,
    SurfaceSpace: TryInto<i32>,
{
    fn handle_event(&mut self, event: WindowEvent, context: &EventContext) -> Option<WindowEvent> {
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    ElementState::Pressed => self.pressed.insert(button),
                    ElementState::Released => self.pressed.remove(&button),
                };
                None
            }
            WindowEvent::CursorMoved { position, .. } => {
                fn convert<SurfaceSpace>(
                    PhysicalPosition { x, y }: PhysicalPosition<SurfaceSpace>,
                ) -> Option<Vector<i32>>
                where
                    SurfaceSpace: TryInto<i32>,
                {
                    Some(Vector::new(x.try_into().ok()?, y.try_into().ok()?))
                }
                self.position = context.estimate_surface_space(position).and_then(convert);
                None
            }
            _ => Some(event),
        }
    }

    fn update(&mut self) {
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

impl<EventContext, SurfaceSpace> InputHandler<WindowEvent, EventContext> for KeyboardMouse
where
    EventContext: middling::EventContext<
        PhysicalPosition<f64>,
        SurfaceSpace = Option<PhysicalPosition<SurfaceSpace>>,
    >,
    SurfaceSpace: TryInto<i32>,
{
    fn handle_event(&mut self, event: WindowEvent, context: &EventContext) -> Option<WindowEvent> {
        let event = self.keyboard.handle_event(event, context)?;
        self.mouse.handle_event(event, context)
    }

    fn update(&mut self) {
        InputHandler::<WindowEvent, EventContext>::update(&mut self.keyboard);
        InputHandler::<WindowEvent, EventContext>::update(&mut self.mouse);
    }
}

/// Cheap input system that handles no input event.
#[derive(Debug, Clone, Copy)]
pub struct NoInput;

impl<Event, EventContext> InputHandler<Event, EventContext> for NoInput {
    fn handle_event(&mut self, event: Event, _: &EventContext) -> Option<Event> {
        Some(event)
    }

    fn update(&mut self) {}
}
