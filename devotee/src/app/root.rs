use devotee_backend::Converter;

use super::config::Config;
use super::context::Context;

/// Root of a `devotee` app.
/// Handles update and render logic.
pub trait Root<Cfg>
where
    Cfg: Config,
    Cfg::Converter: Converter,
{
    /// Update mutably.
    fn update(&mut self, update: &mut Context<Cfg>);

    /// Perform render on provided `RenderTarget`.
    fn render(&self, render: &mut Cfg::RenderTarget);

    /// Handle exit request and optionally forbid it.
    fn handle_exit_request(&mut self) -> ExitPermission {
        ExitPermission::Forbid
    }

    /// Provide palette converter.
    fn converter(&self) -> &Cfg::Converter;

    /// Provide current background color for the window.
    fn background_color(&self) -> <Cfg::Converter as Converter>::Palette;
}

/// Enumeration of exit permission variants.
#[derive(Clone, Copy, Debug)]
pub enum ExitPermission {
    /// Permit exiting the application.
    Allow,
    /// Forbid exiting the application.
    Forbid,
}
