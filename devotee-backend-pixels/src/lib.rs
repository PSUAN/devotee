#![deny(missing_docs)]

//! [Pixels](https://crates.io/crates/pixels)-based backend for the [devotee](https://crates.io/crates/devotee) project.

use std::num::NonZeroU32;

use devotee_backend::winit::dpi::PhysicalPosition;
use devotee_backend::winit::window::Window;
use devotee_backend::{Backend, BackendImage, Converter};
use pixels::wgpu::Color;
use pixels::{Pixels, PixelsBuilder, SurfaceTexture};

/// [Pixels](https://crates.io/crates/pixels)-based backend.
pub struct PixelsBackend {
    pixels: Pixels,
}

impl Backend for PixelsBackend {
    fn new(window: &Window, resolution: (u32, u32), _scale: u32) -> Option<Self> {
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            let builder = PixelsBuilder::new(resolution.0, resolution.1, surface_texture);

            #[cfg(target_arch = "wasm32")]
            {
                pollster::block_on(builder.build_async())
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                builder.build()
            }
        }
        .ok()?;

        Some(PixelsBackend { pixels })
    }

    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Option<()> {
        self.pixels.resize_surface(width.into(), height.into()).ok()
    }

    fn draw_image<'a, P: 'a, I>(
        &mut self,
        image: &'a dyn BackendImage<'a, P, Iterator = I>,
        converter: &dyn Converter<Palette = P>,
        _window: &Window,
        background: u32,
    ) -> Option<()>
    where
        I: Iterator<Item = &'a P>,
    {
        let r = background & 0x00ff0000 >> 16;
        let g = background & 0x0000ff00 >> 8;
        let b = background & 0x000000ff;

        self.pixels.clear_color(Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: 1.0,
        });

        for (chunk, pixel) in self
            .pixels
            .frame_mut()
            .chunks_exact_mut(4)
            .zip(image.pixels())
        {
            let argb = converter.convert(pixel);
            let r = ((argb & 0x00ff0000) >> 16) as u8;
            let g = ((argb & 0x0000ff00) >> 8) as u8;
            let b = (argb & 0x000000ff) as u8;
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = 0xff;
        }

        self.pixels.render().ok()
    }

    fn window_pos_to_inner(
        &self,
        position: PhysicalPosition<f64>,
        _window: &Window,
        _resolution: (u32, u32),
    ) -> Result<(i32, i32), (i32, i32)> {
        self.pixels
            .window_pos_to_pixel((position.x as f32, position.y as f32))
            .map(|(a, b)| (a as i32, b as i32))
            .map_err(|(a, b)| (a as i32, b as i32))
    }
}
