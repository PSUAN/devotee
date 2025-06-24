use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use devotee_backend::middling::{
    Fill, Surface, TexelDesignatorMut, TexelDesignatorRef, TexelMut, TexelRef,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::Window;

pub(crate) fn estimate_render_window_position_scale(
    surface_size: PhysicalSize<u32>,
    scale: ScaleMode,
    render_window_size: PhysicalSize<u32>,
) -> (PhysicalPosition<u32>, u32) {
    let scale = match scale {
        ScaleMode::Fixed(scale) => scale.get(),
        ScaleMode::Auto => (surface_size.width / render_window_size.width)
            .min(surface_size.height / render_window_size.height)
            .max(1),
    };

    (
        PhysicalPosition::new(
            surface_size
                .width
                .saturating_sub(render_window_size.width * scale)
                / 2,
            surface_size
                .height
                .saturating_sub(render_window_size.height * scale)
                / 2,
        ),
        scale,
    )
}

fn filter_coords(
    x: u32,
    y: u32,
    window: (PhysicalPosition<u32>, PhysicalSize<u32>),
) -> Option<(u32, u32)> {
    if x >= window.1.width || y >= window.1.height {
        None
    } else {
        Some((x, y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(super) enum ScaleMode {
    #[default]
    Auto,
    Fixed(NonZeroU32),
}

/// Render surface implementation.
pub struct SoftSurface<'a> {
    internal: softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>,
    surface_size: PhysicalSize<u32>,
    scale: u32,
    render_window: (PhysicalPosition<u32>, PhysicalSize<u32>),
}

impl<'a> SoftSurface<'a> {
    pub(super) fn new(
        internal: softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>,
        surface_size: PhysicalSize<u32>,
        scale: ScaleMode,
        render_window_size: PhysicalSize<u32>,
    ) -> Self {
        let (render_window_position, scale) =
            estimate_render_window_position_scale(surface_size, scale, render_window_size);
        let render_window_size = PhysicalSize::new(
            render_window_size.width.min(surface_size.width),
            render_window_size.height.min(surface_size.height),
        );
        let render_window = (render_window_position, render_window_size);
        Self {
            internal,
            surface_size,
            scale,
            render_window,
        }
    }

    pub(super) fn clear(&mut self, color: u32) -> Result<(), softbuffer::SoftBufferError> {
        self.internal.fill(color);
        Ok(())
    }

    pub(super) fn present(self) -> Result<(), softbuffer::SoftBufferError> {
        self.internal.present()
    }
}

impl Fill for SoftSurface<'_> {
    fn fill_from(&mut self, data: &[Self::Texel]) {
        let start_x = self.render_window.0.x;
        let start_y = self.render_window.0.y;

        for y in 0..self.render_window.1.height {
            for x in 0..self.render_window.1.width {
                if let Some(pixel) = data.get((x + y * self.render_window.1.width) as usize) {
                    for internal_y in 0..self.scale {
                        let index = ((start_x + x * self.scale) as usize)
                            + ((start_y + internal_y + (y * self.scale)) * self.surface_size.width)
                                as usize;
                        if let Some(buffer) =
                            self.internal.get_mut(index..(index + self.scale as usize))
                        {
                            buffer.fill(*pixel);
                        }
                    }
                }
            }
        }
    }
}

impl TexelDesignatorRef<'_> for SoftSurface<'_> {
    type TexelRef = TexelReader;
}

impl<'t> TexelDesignatorMut<'t> for SoftSurface<'_> {
    type TexelMut = TexelWriter<'t>;
}

impl Surface for SoftSurface<'_> {
    type Texel = u32;

    fn texel(&self, x: u32, y: u32) -> Option<TexelRef<'_, Self>> {
        let (x, y) = filter_coords(x, y, self.render_window)?;
        TexelReader::try_new(
            x,
            y,
            self.render_window.0,
            self.surface_size,
            self.scale,
            self.internal.deref(),
        )
    }

    fn texel_mut(&mut self, x: u32, y: u32) -> Option<TexelMut<'_, Self>> {
        let (x, y) = filter_coords(x, y, self.render_window)?;
        TexelWriter::try_new(
            x,
            y,
            self.render_window.0,
            self.surface_size,
            self.scale,
            self.internal.deref_mut(),
        )
    }

    unsafe fn unsafe_texel(&self, x: u32, y: u32) -> TexelRef<'_, Self> {
        TexelReader::new(
            x,
            y,
            self.render_window.0,
            self.surface_size,
            self.scale,
            self.internal.deref(),
        )
    }

    unsafe fn unsafe_texel_mut(&mut self, x: u32, y: u32) -> TexelMut<'_, Self> {
        TexelWriter::new(
            x,
            y,
            self.render_window.0,
            self.surface_size,
            self.scale,
            self.internal.deref_mut(),
        )
    }

    fn clear(&mut self, value: Self::Texel) {
        for y in self.render_window.0.y
            ..(self.render_window.0.y + self.render_window.1.height * self.scale)
        {
            if let Some(slice) = self.internal.get_mut(
                ((self.render_window.0.x + y * self.surface_size.width) as usize)
                    ..((self.render_window.0.x
                        + self.render_window.1.width * self.scale
                        + y * self.surface_size.width) as usize),
            ) {
                slice.fill(value);
            }
        }
    }

    fn width(&self) -> u32 {
        self.render_window.1.width
    }

    fn height(&self) -> u32 {
        self.render_window.1.height
    }
}

pub struct TexelReader {
    cache: u32,
}

impl TexelReader {
    fn new(
        x: u32,
        y: u32,
        window_start: PhysicalPosition<u32>,
        surface_size: PhysicalSize<u32>,
        scale: u32,
        buffer: &[u32],
    ) -> Self {
        let cache =
            buffer[(window_start.x + x * scale + (y * scale + window_start.y) * surface_size.width)
                as usize];
        Self { cache }
    }

    fn try_new(
        x: u32,
        y: u32,
        window_start: PhysicalPosition<u32>,
        surface_size: PhysicalSize<u32>,
        scale: u32,
        buffer: &[u32],
    ) -> Option<Self> {
        {
            let (x, y) = (window_start.x + x * scale, window_start.y + y * scale);
            if x >= surface_size.width || y >= surface_size.height {
                return None;
            }
        }
        let cache = *buffer.get(
            (window_start.x + x * scale + (y * scale + window_start.y) * surface_size.width)
                as usize,
        )?;
        Some(Self { cache })
    }
}

impl Deref for TexelReader {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

pub struct TexelWriter<'a> {
    start_x: u32,
    start_y: u32,
    surface_size: PhysicalSize<u32>,
    scale: u32,
    buffer: &'a mut [u32],
    cache: u32,
}

impl<'a> TexelWriter<'a> {
    fn new(
        x: u32,
        y: u32,
        window_start: PhysicalPosition<u32>,
        surface_size: PhysicalSize<u32>,
        scale: u32,
        buffer: &'a mut [u32],
    ) -> Self {
        let start_x = window_start.x + x * scale;
        let start_y = window_start.y + y * scale;
        let cache = buffer[(start_x + start_y * surface_size.width) as usize];

        Self {
            start_x,
            start_y,
            surface_size,
            scale,
            buffer,
            cache,
        }
    }

    fn try_new(
        x: u32,
        y: u32,
        window_start: PhysicalPosition<u32>,
        surface_size: PhysicalSize<u32>,
        scale: u32,
        buffer: &'a mut [u32],
    ) -> Option<Self> {
        let start_x = window_start.x + x * scale;
        let start_y = window_start.y + y * scale;
        {
            let (x, y) = (start_x, start_y);
            if x >= surface_size.width || y >= surface_size.height {
                return None;
            }
        }
        let cache = *buffer.get((start_x + start_y * surface_size.width) as usize)?;

        Some(Self {
            start_x,
            start_y,
            surface_size,
            scale,
            buffer,
            cache,
        })
    }
}

impl Deref for TexelWriter<'_> {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl DerefMut for TexelWriter<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

impl Drop for TexelWriter<'_> {
    fn drop(&mut self) {
        let start_x = self.start_x;
        let end_x = (start_x + self.scale).min(self.surface_size.width);
        for y in self.start_y..(self.start_y + self.scale) {
            let slice_y = y * self.surface_size.width;
            self.buffer[((start_x + slice_y) as usize)..((end_x + slice_y) as usize)]
                .fill(self.cache)
        }
    }
}
