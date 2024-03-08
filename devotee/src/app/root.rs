use devotee_backend::Converter;

use super::AppContext;

pub trait Root {
    type Input;
    type Converter: Converter;
    type RenderSurface;

    fn update(&mut self, update: AppContext<Self::Input>);
    fn render(&self, render: &mut Self::RenderSurface);
    fn converter(&self) -> Self::Converter;
    fn background_color(&self) -> <Self::Converter as Converter>::Data;
    fn pause(&mut self) {}
    fn resume(&mut self) {}
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
