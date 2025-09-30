/// So-so display surface trait for reuse purposes.
pub trait Surface {
    /// Specific texel type.
    type Texel;

    /// Get texel given its coordinates.
    fn texel(&self, x: u32, y: u32) -> Option<Self::Texel>;

    /// Set texel given its coordinates.
    fn set_texel(&mut self, x: u32, y: u32, value: Self::Texel);

    /// Get texel given its coordinates unsafely.
    ///
    /// # Safety
    ///
    /// - `x` must be in range [`0`, `width - 1`];
    /// - `y` must be in range [`0`, `height - 1`];
    unsafe fn texel_unchecked(&self, x: u32, y: u32) -> Self::Texel {
        self.texel(x, y).unwrap()
    }

    /// Set texel value given its coordinates unsafely.
    ///
    /// # Safety
    ///
    /// - `x` must be in range [`0`, `width - 1`];
    /// - `y` must be in range [`0`, `height - 1`];
    unsafe fn set_texel_unchecked(&mut self, x: u32, y: u32, value: Self::Texel) {
        self.set_texel(x, y, value)
    }

    /// Clear the surface with the given color.
    fn clear(&mut self, value: Self::Texel);

    /// Get surface width.
    fn width(&self) -> u32;

    /// Get surface height.
    fn height(&self) -> u32;
}

/// The surface that can be filled directly.
pub trait Fill: Surface {
    /// Fill the surface from the provided `data`.
    fn fill_from(&mut self, data: &[Self::Texel]);
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
