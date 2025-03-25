#![deny(missing_docs)]

//! Set of definitions for backend-y stuff of the devotee project.

/// So-so middleware implementation.
pub mod middling;

/// Middleware is an adapter between a backend and an application itself.
pub trait Middleware<Init, UpdateContext, Surface, Event, EventContext, Control> {
    /// Handle the initialization event.
    fn on_init(&mut self, init: &mut Init);

    /// Handle the update tick.
    fn on_update(&mut self, context: &mut UpdateContext);

    /// Handle render call, draw on the provided surface.
    fn on_render(&mut self, surface: &mut Surface);

    /// Handle event originated from the backend.
    fn on_event(
        &mut self,
        event: Event,
        event_context: &EventContext,
        event_control: &mut Control,
    ) -> Option<Event>;
}
