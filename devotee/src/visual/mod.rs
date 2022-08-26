use color::Color;

/// Image with dimensions unknown at compile-time.
pub mod canvas;
/// Color system used in `devotee`.
pub mod color;
/// Image with compile-time known dimensions.
pub mod sprite;

/// Drawing traits prelude.
pub mod prelude {
    pub use super::color::Color;
    pub use super::{draw, paint};
    pub use super::{Draw, Image, Line, Pixel, PixelMod, Rect};
}

/// Mapper function accepts `x` and `y` coordinates and pixel value.
pub type Mapper<P> = dyn FnMut(i32, i32, P) -> P;

/// Helper paint function for pixel value override.
pub fn paint<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone,
{
    move |_, _, _| value.clone()
}

/// Helper draw function for pixel value combining.
pub fn draw<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone + Color,
{
    move |_, _, pixel| Color::mix(pixel, value.clone())
}

/// General drawing trait.
pub trait Draw {
    /// Pixel type of this drawable.
    type Pixel;

    /// Width of this drawable.
    fn width(&self) -> i32;
    /// Height of this drawable.
    fn height(&self) -> i32;
    /// Clear this drawable with given pixel value.
    fn clear(&mut self, clear_color: Self::Pixel);
}

/// Provide access to specific pixel values.
pub trait Pixel<I>: Draw {
    /// Get reference to pixel.
    fn pixel(&self, position: I) -> Option<&Self::Pixel>;
    /// Get mutable reference to pixel.
    fn pixel_mut(&mut self, position: I) -> Option<&mut Self::Pixel>;
}

/// Provide pixel manipulation access.
pub trait PixelMod<I, F>: Draw {
    /// Use provided function on pixel at given position.
    fn mod_pixel(&mut self, position: I, function: F);
}

/// Provide unsafe access to specific pixel values.
pub trait UnsafePixel<I>: Draw {
    /// Get reference to pixel.
    ///
    /// # Safety
    /// - `position` must be in the `[0, (width, height))` range.
    unsafe fn pixel(&self, position: I) -> &Self::Pixel;

    /// Get mutable reference to pixel.
    ///
    /// # Safety
    /// - `position` must be in the `[0, (width, height))` range.
    unsafe fn pixel_mut(&mut self, position: I) -> &mut Self::Pixel;
}

/// Allow function application on a line in the given (inclusive) range.
pub trait Line<I, F>: Draw {
    /// Use provided function on each pixel in a line.
    fn line(&mut self, from: I, to: I, function: F);
}

/// Allow function application on a rectangle in the given (exclusive) range.
pub trait Rect<I, F>: Draw {
    /// Use provided function on each pixel in a rectangle.
    fn filled_rect(&mut self, from: I, to: I, function: F);

    /// Use provided function on each pixel in rectangle bounds.
    fn rect(&mut self, from: I, to: I, function: F);
}

/// Apply image using provided mapper function.
pub trait Image<I, O, U, F>: UnsafePixel<I> {
    /// Use provided function and given image on this drawable.
    fn image(&mut self, at: I, image: &U, function: F);
}
