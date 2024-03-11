use devotee_backend::Converter;

use super::AppContext;

/// App's root trait.
pub trait Root {
    /// Input system handled by the root.
    type Input;

    /// Pixel data converter.
    type Converter: Converter;

    /// Render surface to render on.
    type RenderSurface;

    /// Handle update event.
    fn update(&mut self, context: AppContext<Self::Input>);

    /// Handle rendering on the surface.
    fn render(&self, surface: &mut Self::RenderSurface);

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
