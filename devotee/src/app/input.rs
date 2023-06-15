pub use winit::event;

use super::window::Window;

/// Keyboard and mouse input module.
pub mod key_mouse;

/// Input trait.
/// Specifies input storing.
pub trait Input {
    /// Handle frame change.
    fn next_frame(&mut self);
    /// Register `winit` event.
    /// Return `None` if the event is consumed.
    fn consume_window_event<'a>(
        &mut self,
        event: event::WindowEvent<'a>,
        window: &Window,
    ) -> Option<event::WindowEvent<'a>>;
}
