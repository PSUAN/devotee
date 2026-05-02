/// So-so display surface trait for reuse purposes.
pub trait Surface {
    /// Specific texel type.
    type Texel;

    /// Get texel given its coordinates.
    fn texel(&self, x: u32, y: u32) -> Option<Self::Texel>;

    /// Set texel given its coordinates.
    fn set_texel(&mut self, x: u32, y: u32, value: Self::Texel);

    /// Clear the surface with the given color.
    fn clear(&mut self, value: Self::Texel);

    /// Get surface width.
    fn width(&self) -> u32;

    /// Get surface height.
    fn height(&self) -> u32;
}

/// The surface that can be filled directly.
pub trait Fill<I>: Surface {
    /// Fill the surface from the provided `data`.
    fn fill_from(&mut self, data: I);
}

/// Input handler trait with optional input caching.
pub trait InputHandler<Event, EventContext> {
    /// Handle input event, optionally consume it.
    fn handle_event(&mut self, event: Event, event_context: &EventContext) -> Option<Event>;

    /// Update internal state over tick event.
    fn update(&mut self);
}

/// Event context generalization.
pub trait EventContext<EventSpace> {
    /// Surface space coordinates representation.
    type SurfaceSpace;

    /// Recalculate from the event space coordinates into the surface space coordinates.
    fn estimate_surface_space(&self, event_space: EventSpace) -> Self::SurfaceSpace;
}
