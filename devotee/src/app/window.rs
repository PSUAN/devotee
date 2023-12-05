use std::num::NonZeroU32;

use devotee_backend::winit::dpi::{PhysicalPosition, PhysicalSize};
use devotee_backend::winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use devotee_backend::winit::platform::web::WindowExtWebSys;
use devotee_backend::winit::window::{Fullscreen, Window as WinitWindow, WindowBuilder};
use devotee_backend::{Backend, BackendImage};
#[cfg(feature = "back-pixels")]
use devotee_backend_pixels::PixelsBackend;
#[cfg(feature = "back-softbuffer")]
use devotee_backend_softbuffer::SoftbufferBackend;

use super::{Config, Setup};
use crate::util::vector::Vector;
use crate::visual::color::Converter;
use crate::visual::Image;

pub use devotee_backend::winit;

pub(super) type WindowCommand = Box<dyn FnOnce(&mut Window)>;

#[cfg(feature = "back-softbuffer")]
type Back = SoftbufferBackend;

#[cfg(feature = "back-pixels")]
type Back = PixelsBackend;

/// The application window.
pub struct Window {
    window: WinitWindow,
    back: Back,
    resolution: Vector<u32>,
    background: u32,
}

impl Window {
    pub(super) fn with_setup<Cfg>(event_loop: &EventLoop<()>, setup: &Setup<Cfg>) -> Option<Self>
    where
        Cfg: Config,
        Cfg::RenderTarget: Image,
        Cfg::Converter: Converter,
    {
        let resolution = Vector::new(
            setup.render_target.width() as u32,
            setup.render_target.height() as u32,
        );
        if resolution.x() == 0 || resolution.y() == 0 {
            return None;
        }
        if setup.scale == 0 {
            return None;
        }
        let window_size = resolution * setup.scale;

        let window = {
            let builder = WindowBuilder::new()
                .with_min_inner_size(PhysicalSize::new(resolution.x(), resolution.y()))
                .with_inner_size(PhysicalSize::new(window_size.x(), window_size.y()))
                .with_fullscreen(if setup.fullscreen {
                    Some(Fullscreen::Borderless(None))
                } else {
                    None
                })
                .with_title(&setup.title);
            builder.build(event_loop).ok()?
        };
        window.set_cursor_visible(false);
        #[cfg(target_arch = "wasm32")]
        {
            let document = web_sys::window()?.document()?;
            if let Some(canvas_holder) =
                document.get_element_by_id(setup.element_id.unwrap_or("devoteeCanvasHolder"))
            {
                let _ = canvas_holder.append_child(&web_sys::Element::from(window.canvas()));
            } else {
                let _ = document
                    .body()?
                    .append_child(&web_sys::Element::from(window.canvas()));
            }
        }

        let back = Backend::new(&window, resolution.split(), setup.scale)?;

        let background = Cfg::converter().convert(&Cfg::background_color());

        Some(Window {
            window,
            back,
            resolution,
            background,
        })
    }

    pub(super) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(super) fn resize_surface(&mut self, width: NonZeroU32, height: NonZeroU32) -> Option<()> {
        self.back.resize(width, height)
    }

    /// Get window pixel resolution.
    pub fn resolution(&self) -> Vector<u32> {
        self.resolution
    }

    /// Set window to fullscreen mode.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.window.set_fullscreen(None);
        }
    }

    /// Check if the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.window.fullscreen().is_some()
    }

    pub(super) fn apply(&mut self, commands: &mut Vec<WindowCommand>) {
        for command in commands.drain(..) {
            command(self)
        }
    }

    pub(super) fn draw_image<'a, P: 'a, I>(
        &mut self,
        image: &'a dyn BackendImage<'a, P, Iterator = I>,
        converter: &dyn Converter<Palette = P>,
    ) -> Option<()>
    where
        I: Iterator<Item = &'a P>,
    {
        self.back
            .draw_image(image, converter, &self.window, self.background)
    }

    /// Recalculate raw window position into camera-related.
    pub fn window_pos_to_inner(
        &self,
        position: PhysicalPosition<f64>,
    ) -> Result<Vector<i32>, Vector<i32>> {
        self.back
            .window_pos_to_inner(position, &self.window, self.resolution.split())
            .map(Into::into)
            .map_err(Into::into)
    }
}
