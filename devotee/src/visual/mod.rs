use crate::util::getter::Getter;
use crate::util::vector::Vector;
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
    pub use super::{draw, paint, printer, stamp};
    pub use super::{Draw, Image, Line, Pixel, PixelMod, Rect, Text, Tilemap};
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

/// Helper printer mapper for the `Text` trait.
pub fn printer<U>() -> impl FnMut(char, &U) -> Vector<i32>
where
    U: Draw,
{
    let mut column = 0;
    let mut line = 0;
    move |code_point, representation| {
        let result = (column, line).into();
        if code_point == '\n' {
            line += representation.height();
            column = 0;
        } else {
            column += representation.width();
        }
        result
    }
}

/// Helper stamper mapper for image-image mapping.
pub fn stamp<P>() -> impl FnMut(i32, i32, P, i32, i32, P) -> P
where
    P: Clone + Color,
{
    move |_, _, pixel, _, _, other| pixel.mix(other)
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
pub trait Image<I, U, F>: UnsafePixel<I> {
    /// Use provided function and given image on this drawable.
    fn image(&mut self, at: I, image: &U, function: F);
}

/// Apply multiple images provided spatial and color mapper functions.
pub trait Tilemap<I, U, F, M>: Image<I, U, F> {
    /// Use provided spatial mapper, tiles and color mapper function.
    fn tilemap(
        &mut self,
        at: I,
        mapper: M,
        tiles: &dyn Getter<Index = usize, Item = U>,
        tile_data: &mut dyn Iterator<Item = usize>,
        function: F,
    );
}

/// Draw text using mapper, font and the text itself.
pub trait Text<I, U, F, M>: Image<I, U, F> {
    /// Use provided spatial mapper, tiles and color mapper function to draw text.
    fn text(
        &mut self,
        at: I,
        mapper: M,
        font: &dyn Getter<Index = char, Item = U>,
        text: &str,
        function: F,
    );
}
