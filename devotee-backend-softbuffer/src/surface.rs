use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use devotee_backend::middling::{
    Surface, TexelDesignatorMut, TexelDesignatorRef, TexelMut, TexelRef,
};
use softbuffer::Buffer;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::Window;

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
///
/// # Note
/// Due to the implementation limitations, surface lies about values being read in immutable mode.
/// It will always return `0` value for the pixel color in such cases.
pub struct SoftSurface<'a> {
    internal: &'a mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
    surface_size: PhysicalSize<u32>,
    scale: u32,
    render_window: (PhysicalPosition<u32>, PhysicalSize<u32>),
}

impl<'a> SoftSurface<'a> {
    pub(super) fn new(
        internal: &'a mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
        surface_size: PhysicalSize<u32>,
        scale: ScaleMode,
        render_window_size: PhysicalSize<u32>,
    ) -> Self {
        let scale = match scale {
            ScaleMode::Fixed(scale) => scale.get(),
            ScaleMode::Auto => (surface_size.width / render_window_size.width)
                .min(surface_size.height / render_window_size.height)
                .max(1),
        };

        let window = (
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
            render_window_size,
        );
        Self {
            internal,
            surface_size,
            scale,
            render_window: window,
        }
    }

    pub(super) fn clear(&mut self, color: u32) -> Result<(), softbuffer::SoftBufferError> {
        self.internal.buffer_mut()?.fill(color);
        Ok(())
    }

    pub(super) fn present(&mut self) -> Result<(), softbuffer::SoftBufferError> {
        self.internal.buffer_mut()?.present()
    }

    pub(super) fn render_window_position(&self) -> PhysicalPosition<u32> {
        self.render_window.0
    }

    pub(super) fn render_window_size(&self) -> PhysicalSize<u32> {
        self.render_window.1
    }

    pub(super) fn render_window_scale(&self) -> u32 {
        self.scale
    }
}

impl TexelDesignatorRef<'_> for SoftSurface<'_> {
    type TexelRef = DummyTexelReader;
}

impl<'t> TexelDesignatorMut<'t> for SoftSurface<'_> {
    type TexelMut = TexelWriter<'t>;
}

impl Surface for SoftSurface<'_> {
    type Texel = u32;

    fn texel(&self, x: u32, y: u32) -> Option<TexelRef<'_, Self>> {
        let (x, y) = filter_coords(x, y, self.render_window)?;
        if x <= (self.surface_size.width - self.render_window.0.x) / self.scale
            && y <= (self.surface_size.height - self.render_window.0.y) / self.scale
        {
            Some(DummyTexelReader)
        } else {
            None
        }
    }

    fn texel_mut(&mut self, x: u32, y: u32) -> Option<TexelMut<'_, Self>> {
        let (x, y) = filter_coords(x, y, self.render_window)?;
        TexelWriter::try_new(
            x,
            y,
            self.render_window.0,
            self.surface_size,
            self.scale,
            self.internal,
        )
    }

    fn clear(&mut self, value: Self::Texel) {
        if let Ok(mut buffer) = self.internal.buffer_mut() {
            for y in self.render_window.0.y
                ..(self.render_window.0.y + self.render_window.1.height * self.scale)
            {
                if let Some(slice) = buffer.get_mut(
                    ((self.render_window.0.x + y * self.surface_size.width) as usize)
                        ..((self.render_window.0.x
                            + self.render_window.1.width * self.scale
                            + y * self.surface_size.width) as usize),
                ) {
                    slice.fill(value);
                }
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

pub struct DummyTexelReader;

impl Deref for DummyTexelReader {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &0
    }
}

pub struct TexelWriter<'a> {
    x: u32,
    y: u32,
    window_start: PhysicalPosition<u32>,
    surface_size: PhysicalSize<u32>,
    scale: u32,
    buffer: Buffer<'a, Rc<Window>, Rc<Window>>,
    cache: u32,
}

impl<'a> TexelWriter<'a> {
    fn try_new(
        x: u32,
        y: u32,
        window_start: PhysicalPosition<u32>,
        surface_size: PhysicalSize<u32>,
        scale: u32,
        surface: &'a mut softbuffer::Surface<Rc<Window>, Rc<Window>>,
    ) -> Option<Self> {
        let buffer = surface.buffer_mut().ok()?;

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

        Some(Self {
            x,
            y,
            window_start,
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
        let start_x = self.window_start.x + self.x * self.scale;
        let end_x = (self.window_start.x + (self.x + 1) * self.scale).min(self.surface_size.width);
        for y in (self.y * self.scale + self.window_start.y)
            ..((self.y + 1) * self.scale + self.window_start.y)
        {
            let slice_y = y * self.surface_size.width;
            self.buffer[((start_x + slice_y) as usize)..((end_x + slice_y) as usize)]
                .fill(self.cache)
        }
    }
}
