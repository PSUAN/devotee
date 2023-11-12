#![deny(missing_docs)]

//! [Softbuffer](https://crates.io/crates/softbuffer)-based backend for the [devotee](https://crates.io/crates/devotee) project.

use std::num::NonZeroU32;

use devotee_backend::winit::dpi::PhysicalPosition;
use devotee_backend::winit::window::Window;
use devotee_backend::{Backend, BackendImage, Converter};
use softbuffer::{Context, Surface};

/// [Softbuffer](https://crates.io/crates/softbuffer)-based backend.
pub struct SoftbufferBackend {
    surface: Surface,
}

impl Backend for SoftbufferBackend {
    fn new(window: &Window, resolution: (u32, u32), scale: u32) -> Option<Self> {
        // SAFETY: context and window are stored in the same struct, so the window will be valid for all context lifetime.
        let context = unsafe { Context::new(&window) }.ok()?;
        // SAFETY: surface, context and window are in the same struct.
        // Surface will be valid for a proper lifetime.
        let mut surface = unsafe { Surface::new(&context, &window) }.ok()?;
        // SAFETY: arguments won't be zero, checked in advance.
        unsafe {
            surface.resize(
                NonZeroU32::new_unchecked(resolution.0 * scale),
                NonZeroU32::new_unchecked(resolution.1 * scale),
            )
        }
        .ok()?;

        Some(SoftbufferBackend { surface })
    }

    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Option<()> {
        self.surface.resize(width, height).ok()
    }

    fn draw_image<'a, P: 'a, I>(
        &mut self,
        image: &'a dyn BackendImage<'a, P, Iterator = I>,
        converter: &dyn Converter<Palette = P>,
        window: &Window,
        background: u32,
    ) -> Option<()>
    where
        I: Iterator<Item = &'a P>,
    {
        let mut buffer = self.surface.buffer_mut().ok()?;

        let surface_size = window.inner_size();

        if buffer.len() != (surface_size.width * surface_size.height) as usize {
            return Some(());
        }

        buffer.fill(background);

        let scale_x = surface_size.width / image.width();
        let scale_y = surface_size.height / image.height();

        let minimal_scale = scale_x.min(scale_y);

        if minimal_scale < 1 {
        } else {
            let start_x = (surface_size.width - image.width() * minimal_scale) as usize / 2;
            let start_y = (surface_size.height - image.height() * minimal_scale) as usize / 2;

            for y in 0..image.height() {
                for x in 0..image.width() {
                    // Safety: we are sure that we are in a proper range due to for loops proper arguments.
                    let pixel = unsafe { image.pixel_unsafe(x, y) };
                    let rgb = converter.convert(pixel);

                    for iy in 0..minimal_scale {
                        let index = (start_x + (x * minimal_scale) as usize)
                            + (iy as usize + start_y + (y * minimal_scale) as usize)
                                * surface_size.width as usize;

                        buffer[index..index + minimal_scale as usize].fill(rgb);
                    }
                }
            }
        }

        buffer.present().ok()
    }

    fn window_pos_to_inner(
        &self,
        position: PhysicalPosition<f64>,
        window: &Window,
        resolution: (u32, u32),
    ) -> Result<(i32, i32), (i32, i32)> {
        let size = window.inner_size();
        let scale_x = size.width / resolution.0;
        let scale_y = size.height / resolution.1;

        let minimal_scale = scale_x.min(scale_y);

        if minimal_scale < 1 {
            Err((0, 0))
        } else {
            let position = (position.x as i32, position.y as i32);
            let start_x = ((size.width - resolution.0 * minimal_scale) / 2) as i32;
            let start_y = ((size.height - resolution.1 * minimal_scale) / 2) as i32;

            let position = (
                (position.0 - start_x) / minimal_scale as i32,
                (position.1 - start_y) / minimal_scale as i32,
            );

            if position.0 < 0
                || position.0 >= resolution.0 as i32
                || position.1 < 0
                || position.1 >= resolution.1 as i32
            {
                Err(position)
            } else {
                Ok(position)
            }
        }
    }
}
