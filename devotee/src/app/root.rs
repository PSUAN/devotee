use devotee_backend::Converter;

/// App's root trait.
pub trait Root<Init, Context> {
    /// Pixel data converter.
    type Converter: Converter;

    /// Render surface to render on.
    type RenderSurface;

    /// Handle initialization event.
    fn init(&mut self, init: &mut Init);

    /// Handle update event.
    fn update(&mut self, context: &mut Context);

    /// Handle rendering on the surface.
    fn render(&mut self, surface: &mut Self::RenderSurface);

    /// Get converter to convert Render Surface pixels into `u32` values.
    fn converter(&self) -> Self::Converter;

    /// Handle pause event.
    fn pause(&mut self) {}

    /// Handle resume event.
    fn resume(&mut self) {}

    /// Handle exit request and give optional permission to shut down the App.
    fn handle_exit_request(&mut self) -> ExitPermission {
        ExitPermission::Allow
    }
}

/// Enumeration of exit permission variants.
#[derive(Clone, Copy, Debug)]
pub enum ExitPermission {
    /// Permit exiting the application.
    Allow,
    /// Forbid exiting the application.
    Forbid,
}
