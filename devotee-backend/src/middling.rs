use crate::Middleware;

/// Input-caching middleware implementation.
pub struct MiddlingMiddleware<App, Input> {
    application: App,
    input: Input,
}

impl<App, Input> MiddlingMiddleware<App, Input> {
    /// Create new middleware instance.
    pub fn new(application: App, input: Input) -> Self {
        Self { application, input }
    }
}

impl<App, Init, UpdateContext, Input, Surface, Event, EventContext, Control>
    Middleware<Init, UpdateContext, Surface, Event, EventContext, Control>
    for MiddlingMiddleware<App, Input>
where
    App: Application<Init, UpdateContext, Input, Surface>,
    Input: InputHandler<Event, EventContext>,
{
    fn on_init(&mut self, init: &mut Init) {
        self.application.init(init);
    }

    fn on_update(&mut self, context: &mut UpdateContext) {
        self.application.update(context, &self.input);
        self.input.update();
    }

    fn on_render(&mut self, surface: &mut Surface) {
        self.application.render(surface);
    }

    fn on_event(
        &mut self,
        event: Event,
        event_context: &EventContext,
        _: &mut Control,
    ) -> Option<Event> {
        self.input.handle_event(event, event_context)
    }
}

/// So-so application trait for reuse purposes.
pub trait Application<Init, Context, Input, Surface> {
    /// Handle an initialization event.
    fn init(&mut self, init: &mut Init);

    /// Process an update event.
    fn update(&mut self, context: &mut Context, input: &Input);

    /// Handle a render event.
    fn render(&mut self, surface: &mut Surface);
}

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

    /// Clear the surface with the given color.
    fn clear(&mut self, value: Self::Texel);

    /// Get surface width.
    fn width(&self) -> u32;

    /// Get surface height.
    fn height(&self) -> u32;
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
