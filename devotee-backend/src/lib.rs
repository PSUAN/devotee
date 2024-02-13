#![deny(missing_docs)]

//! Backend stuff for the `devotee` engine.

use std::num::NonZeroU32;

use winit::dpi::PhysicalPosition;
use winit::window::Window;

pub use winit;

/// Backend trait.
/// It provides functions to:
///
/// - create new instance;
/// - handle resize events;
/// - to draw output;
/// - recalculate pointer position to canvas space.
pub trait Backend: Sized {
    /// Create new backend instance.
    fn new(window: &Window, resolution: (u32, u32), scale: u32) -> Option<Self>;

    /// Handle resize event.
    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Option<()>;

    /// Draw image on the backend.
    fn draw_image<'a, P: 'a, I>(
        &mut self,
        image: &'a dyn BackendImage<'a, P, Iterator = I>,
        converter: &dyn Converter<Palette = P>,
        window: &Window,
        background: u32,
    ) -> Option<()>
    where
        I: Iterator<Item = &'a P>;

    /// Recalculate pointer position to canvas space.
    fn window_pos_to_inner(
        &self,
        position: PhysicalPosition<f64>,
        window: &Window,
        resolution: (u32, u32),
    ) -> Result<(i32, i32), (i32, i32)>;
}

/// Converter from pallette value to `u32` value.
pub trait Converter {
    /// Palette to convert from.
    type Palette;
    /// Perform conversion.
    /// The output is considered to be `0x00rrggbb`.
    fn convert(&self, color: &Self::Palette) -> u32;
}

/// Trait to generalize images to be displayed on the backend.
pub trait BackendImage<'a, P: 'a> {
    /// Iterator to produce pixel values of the image, row-by-row.
    type Iterator: Iterator<Item = &'a P>;

    /// Get reference to specific pixel.
    ///
    /// # Safety
    /// - `x` must be in bounds `[0; width)`;
    /// - `y` must be in bounds `[0; height)`.
    unsafe fn pixel_unsafe(&self, x: u32, y: u32) -> &P;

    /// Get image width in pixels.
    fn width(&self) -> u32;

    /// Get image height in pixels.
    fn height(&self) -> u32;

    /// Get iterator over pixels.
    ///
    /// The iterator is considered to provide pixels row-by-row.
    fn pixels(&'a self) -> Self::Iterator;
}
