/// Type trait that can point on a texel reference.
pub trait TexelDesignatorRef<'a> {
    /// Texel reference.
    type TexelRef;
}

/// Type trait that can point on a mutable texel reference.
pub trait TexelDesignatorMut<'a> {
    /// Mutable texel reference.
    type TexelMut;
}

/// Helper type to represent texel reference.
pub type TexelRef<'a, This> = <This as TexelDesignatorRef<'a>>::TexelRef;

/// Helper type to represent mutable texel reference.
pub type TexelMut<'a, This> = <This as TexelDesignatorMut<'a>>::TexelMut;

/// So-so display surface trait for reuse purposes.
pub trait Surface: for<'a> TexelDesignatorRef<'a> + for<'a> TexelDesignatorMut<'a> {
    /// Specific texel type.
    type Texel;

    /// Get texel reference given its coordinates.
    fn texel(&self, x: u32, y: u32) -> Option<TexelRef<'_, Self>>;

    /// Get mutable texel reference given its coordinates.
    fn texel_mut(&mut self, x: u32, y: u32) -> Option<TexelMut<'_, Self>>;

    /// Get texel reference given its coordinates unsafely.
    ///
    /// # Safety
    ///
    /// - `x` must be in range [`0`, `width - 1`];
    /// - `y` must be in range [`0`, `height - 1`];
    unsafe fn unsafe_texel(&self, x: u32, y: u32) -> TexelRef<'_, Self> {
        self.texel(x, y).unwrap()
    }

    /// Get mutable texel reference given its coordinates unsafely.
    ///
    /// # Safety
    ///
    /// - `x` must be in range [`0`, `width - 1`];
    /// - `y` must be in range [`0`, `height - 1`];
    unsafe fn unsafe_texel_mut(&mut self, x: u32, y: u32) -> TexelMut<'_, Self> {
        self.texel_mut(x, y).unwrap()
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
