use std::num::NonZeroU32;

use softbuffer::{Context, SoftBufferError, Surface};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::EventLoop;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::{Fullscreen, Window as WinitWindow, WindowBuilder};

use super::{Config, Setup};
use crate::util::vector::Vector;
use crate::visual::color::Converter;
use crate::visual::Image;

pub(super) type WindowCommand = Box<dyn FnOnce(&mut Window)>;

/// The application window.
pub struct Window {
    window: WinitWindow,
    surface: Surface,
    resolution: Vector<u32>,
    background: u32,
}

impl Window {
    pub(super) fn with_setup<Cfg>(event_loop: &EventLoop<()>, setup: &Setup<Cfg>) -> Option<Self>
    where
        Cfg: Config,
        Cfg::RenderTarget: Image,
        Cfg::Converter: Converter<Palette = <Cfg::RenderTarget as Image>::Pixel>,
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
        // SAFETY: context and window are stored in the same struct, so the window will be valid for all context lifetime.
        let context = unsafe { Context::new(&window) }.ok()?;
        // SAFETY: surface, context and window are in the same struct.
        // Surface will be valid for a proper lifetime.
        let mut surface = unsafe { Surface::new(&context, &window) }.ok()?;
        // SAFETY: arguments won't be zero, checked in advance.
        unsafe {
            surface.resize(
                NonZeroU32::new_unchecked(resolution.x() * setup.scale),
                NonZeroU32::new_unchecked(resolution.y() * setup.scale),
            )
        }
        .ok()?;

        let background = setup.background_color;

        Some(Window {
            window,
            surface,
            resolution,
            background,
        })
    }

    pub(super) fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(super) fn resize_surface(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), SoftBufferError> {
        self.surface.resize(width, height)
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

    pub(super) fn draw_image<P, C>(
        &mut self,
        image: &dyn Image<Pixel = P>,
        converter: &C,
    ) -> Result<(), SoftBufferError>
    where
        C: Converter<Palette = P>,
    {
        let mut buffer = self.surface.buffer_mut()?;

        let surface_size = self.window.inner_size();

        if buffer.len() != (surface_size.width * surface_size.height) as usize {
            return Ok(());
        }

        buffer.fill(self.background);

        let scale_x = surface_size.width / image.width() as u32;
        let scale_y = surface_size.height / image.height() as u32;

        let minimal_scale = scale_x.min(scale_y);

        if minimal_scale < 1 {
        } else {
            let start_x = (surface_size.width - image.width() as u32 * minimal_scale) as usize / 2;
            let start_y =
                (surface_size.height - image.height() as u32 * minimal_scale) as usize / 2;

            for y in 0..image.height() {
                for x in 0..image.width() {
                    // Safety: we are sure that we are in a proper range due to for loops proper arguments.
                    let pixel = unsafe { image.pixel_unsafe(Vector::new(x, y)) };
                    let rgb = converter.convert(pixel);

                    for iy in 0..minimal_scale {
                        let index = (start_x + (x * minimal_scale as i32) as usize)
                            + (iy as usize + start_y + (y * minimal_scale as i32) as usize)
                                * surface_size.width as usize;

                        buffer[index..index + minimal_scale as usize].fill(rgb);
                    }
                }
            }
        }

        buffer.present()
    }

    /// Recalculate raw window position into camera-related.
    pub fn window_pos_to_inner(
        &self,
        position: PhysicalPosition<f64>,
    ) -> Result<Vector<i32>, Vector<i32>> {
        let size = self.window.inner_size();
        let scale_x = size.width / self.resolution.x();
        let scale_y = size.height / self.resolution.y();

        let minimal_scale = scale_x.min(scale_y);

        if minimal_scale < 1 {
            Err(Vector::new(0, 0))
        } else {
            let position = Vector::new(position.x as i32, position.y as i32);
            let start_x = ((size.width - self.resolution.x() * minimal_scale) / 2) as i32;
            let start_y = ((size.height - self.resolution.y() * minimal_scale) / 2) as i32;

            let position = (position - Vector::new(start_x, start_y)) / minimal_scale as i32;

            if position.x() < 0
                || position.x() >= self.resolution.x() as i32
                || position.y() < 0
                || position.y() >= self.resolution.y() as i32
            {
                Err(position)
            } else {
                Ok(position)
            }
        }
    }
}
