use pixels::Pixels;
use winit::event::WindowEvent;

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
        event: WindowEvent<'a>,
        pixels: &Pixels,
    ) -> Option<WindowEvent<'a>>;
}
