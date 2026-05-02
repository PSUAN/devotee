#![no_std]
#![deny(missing_docs)]

//! Set of definitions for backend-y stuff of the devotee project.

/// So-so middleware implementation.
pub mod middling;

/// Middleware is an adapter between a backend and an application itself.
pub trait Middleware<Init, UpdateContext, OnRender, Event, EventContext> {
    /// Handle the initialization event.
    fn on_init(&mut self, init: &mut Init);

    /// Handle the update tick.
    fn on_update(&mut self, context: &mut UpdateContext);

    /// Handle render call, draw on the provided surface.
    fn on_render(&mut self, surface: &mut OnRender);

    /// Handle the backend event.
    fn on_event(&mut self, event: Event, event_context: &mut EventContext) -> Option<Event>;
}
