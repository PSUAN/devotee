use std::time::Duration;

use super::config::Config;
use super::context::Context;
use super::Constructor;

/// Application setup structure.
/// Describes root node, title, resolution, etc.
pub struct Setup<Cfg>
where
    Cfg: Config,
{
    pub(super) title: String,
    pub(super) update_delay: Duration,
    pub(super) fullscreen: bool,
    pub(super) scale: u32,
    pub(super) render_target: Cfg::RenderTarget,
    pub(super) constructor: Constructor<Cfg::Node, Cfg>,
    #[cfg(target_arch = "wasm32")]
    pub(super) element_id: Option<&'static str>,
    pub(super) pause_on_focus_lost: bool,
    pub(super) input: Cfg::Input,
    pub(super) background_color: [u8; 3],
}

impl<Cfg> Setup<Cfg>
where
    Cfg: Config,
{
    /// Create new setup with given the `RenderTarget`, `Input` and `Node` constructor.
    /// Defaults to 30 frames per second update.
    pub fn new<F>(render_target: Cfg::RenderTarget, input: Cfg::Input, constructor: F) -> Self
    where
        F: 'static + FnOnce(&mut Context<Cfg>) -> Cfg::Node,
    {
        let title = String::new();
        let update_delay = Duration::from_secs_f64(1.0 / 30.0);
        let fullscreen = false;
        let scale = 1;
        let constructor = Box::new(constructor);
        let background_color = [0, 0, 0];
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
            background_color,
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

    /// Set background color for the window.
    pub fn with_background_color(self, background_color: [u8; 3]) -> Self {
        Self {
            background_color,
            ..self
        }
    }
}
