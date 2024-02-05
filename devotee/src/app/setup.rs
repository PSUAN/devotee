use std::marker::PhantomData;
use std::time::Duration;

use super::context::Context;
use super::root::Root;
use super::Constructor;

/// Builder for `Setup`.
#[derive(Default)]
pub struct Builder<Cfg> {
    _phantom: PhantomData<Cfg>,
}

impl<R> Builder<R>
where
    R: Root,
{
    /// Create new builder.
    pub fn new() -> Builder<R> {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Add render target to setup.
    pub fn with_render_target(self, render_target: R::RenderTarget) -> StepRenderTarget<R> {
        StepRenderTarget { render_target }
    }
}

/// Temporary container of setup properties.
pub struct StepRenderTarget<R>
where
    R: Root,
{
    render_target: R::RenderTarget,
}

impl<R> StepRenderTarget<R>
where
    R: Root,
{
    /// Add input system to setup.
    pub fn with_input(self, input: R::Input) -> StepInput<R> {
        StepInput {
            render_target: self.render_target,
            input,
        }
    }
}

/// Temporary container of setup properties.
pub struct StepInput<R>
where
    R: Root,
{
    render_target: R::RenderTarget,
    input: R::Input,
}

impl<R> StepInput<R>
where
    R: Root,
{
    /// Add root constructor to setup.
    pub fn with_root_constructor<F>(self, constructor: F) -> Setup<R>
    where
        F: 'static + FnOnce(&mut Context<R::Input>) -> R,
    {
        Setup::new(self.render_target, self.input, constructor)
    }
}

/// Application setup structure.
/// Describes root node, title, pause behavior, etc.
pub struct Setup<R>
where
    R: Root,
{
    pub(super) title: String,
    pub(super) update_delay: Duration,
    pub(super) fullscreen: bool,
    pub(super) scale: u32,
    pub(super) render_target: R::RenderTarget,
    pub(super) constructor: Constructor<R, R::Input>,
    #[cfg(target_arch = "wasm32")]
    pub(super) element_id: Option<&'static str>,
    pub(super) pause_on_focus_lost: bool,
    pub(super) input: R::Input,
}

impl<R> Setup<R>
where
    R: Root,
{
    fn new<F>(render_target: R::RenderTarget, input: R::Input, constructor: F) -> Self
    where
        F: 'static + FnOnce(&mut Context<R::Input>) -> R,
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
