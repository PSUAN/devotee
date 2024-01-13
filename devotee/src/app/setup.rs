use std::marker::PhantomData;
use std::time::Duration;

use super::config::Config;
use super::context::Context;
use super::Constructor;

/// Builder for `Setup`.
#[derive(Default)]
pub struct Builder<Cfg> {
    _phantom: PhantomData<Cfg>,
}

impl<Cfg> Builder<Cfg>
where
    Cfg: Config,
{
    /// Create new builder.
    pub fn new() -> Builder<Cfg> {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Add render target to setup.
    pub fn with_render_target(self, render_target: Cfg::RenderTarget) -> StepRenderTarget<Cfg> {
        StepRenderTarget { render_target }
    }
}

/// Temporary container of setup properties.
pub struct StepRenderTarget<Cfg>
where
    Cfg: Config,
{
    render_target: Cfg::RenderTarget,
}

impl<Cfg> StepRenderTarget<Cfg>
where
    Cfg: Config,
{
    /// Add input system to setup.
    pub fn with_input(self, input: Cfg::Input) -> StepInput<Cfg> {
        StepInput {
            render_target: self.render_target,
            input,
        }
    }
}

/// Temporary container of setup properties.
pub struct StepInput<Cfg>
where
    Cfg: Config,
{
    render_target: Cfg::RenderTarget,
    input: Cfg::Input,
}

impl<Cfg> StepInput<Cfg>
where
    Cfg: Config,
{
    /// Add root constructor to setup.
    pub fn with_root_constructor<F>(self, constructor: F) -> Setup<Cfg>
    where
        F: 'static + FnOnce(&mut Context<Cfg>) -> Cfg::Root,
    {
        Setup::new(self.render_target, self.input, constructor)
    }
}

/// Application setup structure.
/// Describes root node, title, pause behavior, etc.
pub struct Setup<Cfg>
where
    Cfg: Config,
{
    pub(super) title: String,
    pub(super) update_delay: Duration,
    pub(super) fullscreen: bool,
    pub(super) scale: u32,
    pub(super) render_target: Cfg::RenderTarget,
    pub(super) constructor: Constructor<Cfg::Root, Cfg>,
    #[cfg(target_arch = "wasm32")]
    pub(super) element_id: Option<&'static str>,
    pub(super) pause_on_focus_lost: bool,
    pub(super) input: Cfg::Input,
}

impl<Cfg> Setup<Cfg>
where
    Cfg: Config,
{
    fn new<F>(render_target: Cfg::RenderTarget, input: Cfg::Input, constructor: F) -> Self
    where
        F: 'static + FnOnce(&mut Context<Cfg>) -> Cfg::Root,
    {
        let title = String::new();
        let update_delay = Duration::from_secs_f64(1.0 / 30.0);
        let fullscreen = false;
        let scale = 1;
        let constructor = Box::new(constructor);
        Self {
            title,
            update_delay,
            fullscreen,
            scale,
            render_target,
            constructor,
            #[cfg(target_arch = "wasm32")]
            element_id: None,
            pause_on_focus_lost: true,
            input,
        }
    }

    /// Set application title.
    pub fn with_title<T: Into<String>>(self, title: T) -> Self {
        Self {
            title: title.into(),
            ..self
        }
    }

    /// Set display scale.
    pub fn with_scale(self, scale: u32) -> Self {
        Self { scale, ..self }
    }

    /// Set fullscreen option.
    pub fn with_fullscreen(self, fullscreen: bool) -> Self {
        Self { fullscreen, ..self }
    }

    /// Set update delay.
    pub fn with_update_delay(self, update_delay: Duration) -> Self {
        Self {
            update_delay,
            ..self
        }
    }

    #[cfg(target_arch = "wasm32")]
    /// Set target element id for canvas holding on wasm32 target.
    pub fn with_element_id(self, element_id: &'static str) -> Self {
        let element_id = Some(element_id);
        Self { element_id, ..self }
    }

    /// Set whether to apply pause on focus being lost.
    pub fn with_pause_on_focus_lost(self, pause_on_focus_lost: bool) -> Self {
        Self {
            pause_on_focus_lost,
            ..self
        }
    }
}
