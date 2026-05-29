use std::collections::HashSet;

use devotee_backend::middling;
use winit::dpi;
use winit::event;
use winit::keyboard;

use crate::util::vector::Vector;

pub use winit as reimport;

/// Keyboard-related input system.
#[derive(Clone, Default, Debug)]
pub struct Keyboard {
    pressed: HashSet<keyboard::KeyCode>,
    was_pressed: HashSet<keyboard::KeyCode>,
}

impl Keyboard {
    /// Create new Keyboard input system instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the key is pressed.
    pub fn is_pressed(&self, key: keyboard::KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Check if the key was pressed during the previous tick and not before.
    pub fn just_pressed(&self, key: keyboard::KeyCode) -> bool {
        self.pressed.contains(&key) && !self.was_pressed.contains(&key)
    }

    /// Check if the key was released during the previous tick.
    pub fn just_released(&self, key: keyboard::KeyCode) -> bool {
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

impl<Context> middling::InputHandler<event::WindowEvent, Context> for Keyboard {
    fn handle_event(
        &mut self,
        event: event::WindowEvent,
        _context: &Context,
    ) -> Option<event::WindowEvent> {
        if let event::WindowEvent::KeyboardInput { event, .. } = event {
            if let keyboard::PhysicalKey::Code(code) = event.physical_key {
                match event.state {
                    event::ElementState::Pressed => self.pressed.insert(code),
                    event::ElementState::Released => self.pressed.remove(&code),
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
    pressed: HashSet<event::MouseButton>,
    was_pressed: HashSet<event::MouseButton>,
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
    pub fn is_pressed(&self, button: event::MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    /// Check if the button was pressed during the previous tick and not before.
    pub fn just_pressed(&self, button: event::MouseButton) -> bool {
        self.pressed.contains(&button) && !self.was_pressed.contains(&button)
    }

    /// Check if the button was released during the previous tick.
    pub fn just_released(&self, button: event::MouseButton) -> bool {
        !self.pressed.contains(&button) && self.was_pressed.contains(&button)
    }

    /// Get mouse position.
    pub fn position(&self) -> MousePosition {
        self.position
    }
}

impl<EventContext, SurfaceSpace> middling::InputHandler<event::WindowEvent, EventContext> for Mouse
where
    EventContext: middling::EventContext<
            dpi::PhysicalPosition<f64>,
            SurfaceSpace = Option<dpi::PhysicalPosition<SurfaceSpace>>,
        >,
    SurfaceSpace: TryInto<i32>,
{
    fn handle_event(
        &mut self,
        event: event::WindowEvent,
        context: &EventContext,
    ) -> Option<event::WindowEvent> {
        match event {
            event::WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    event::ElementState::Pressed => self.pressed.insert(button),
                    event::ElementState::Released => self.pressed.remove(&button),
                };
                None
            }
            event::WindowEvent::CursorMoved { position, .. } => {
                fn convert<SurfaceSpace>(
                    dpi::PhysicalPosition { x, y }: dpi::PhysicalPosition<SurfaceSpace>,
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

impl<EventContext, SurfaceSpace> middling::InputHandler<event::WindowEvent, EventContext>
    for KeyboardMouse
where
    EventContext: middling::EventContext<
            dpi::PhysicalPosition<f64>,
            SurfaceSpace = Option<dpi::PhysicalPosition<SurfaceSpace>>,
        >,
    SurfaceSpace: TryInto<i32>,
{
    fn handle_event(
        &mut self,
        event: event::WindowEvent,
        context: &EventContext,
    ) -> Option<event::WindowEvent> {
        let event = self.keyboard.handle_event(event, context)?;
        self.mouse.handle_event(event, context)
    }

    fn update(&mut self) {
        middling::InputHandler::<event::WindowEvent, EventContext>::update(&mut self.keyboard);
        middling::InputHandler::<event::WindowEvent, EventContext>::update(&mut self.mouse);
    }
}

/// Cheap input system that handles no input event.
#[derive(Debug, Clone, Copy)]
pub struct NoInput;

impl<Event, EventContext> middling::InputHandler<Event, EventContext> for NoInput {
    fn handle_event(&mut self, event: Event, _: &EventContext) -> Option<Event> {
        Some(event)
    }

    fn update(&mut self) {}
}
