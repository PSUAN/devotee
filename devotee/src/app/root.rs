use devotee_backend::Converter;

use super::context::Context;

/// Root of a `devotee` app.
/// Handles update and render logic.
pub trait Root {
    /// Input system accepted by this root.
    type Input;
    /// Converter to produce `u32` values from inner palette.
    type Converter: Converter;
    /// Target to render on.
    type RenderTarget;

    /// Update mutably.
    fn update(&mut self, update: &mut Context<Self::Input>);

    /// Perform render on provided `RenderTarget`.
    fn render(&self, render: &mut Self::RenderTarget);

    /// Handle exit request and optionally forbid it.
    /// By default permits exiting.
    fn handle_exit_request(&mut self) -> ExitPermission {
        ExitPermission::Allow
    }

    /// Provide palette converter.
    fn converter(&self) -> &Self::Converter;

    /// Provide current background color for the window.
    fn background_color(&self) -> <Self::Converter as Converter>::Palette;
}

/// Enumeration of exit permission variants.
#[derive(Clone, Copy, Debug)]
pub enum ExitPermission {
    /// Permit exiting the application.
    Allow,
    /// Forbid exiting the application.
    Forbid,
}
