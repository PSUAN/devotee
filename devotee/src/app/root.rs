use super::config::Config;
use super::context::Context;

/// Root of a `devotee` app.
/// Handles update and render logic.
pub trait Root<Cfg>
where
    Cfg: Config,
{
    /// Update mutably.
    fn update(&mut self, update: &mut Context<Cfg>);

    /// Perform render on provided `RenderTarget`.
    fn render(&self, render: &mut Cfg::RenderTarget);
}
