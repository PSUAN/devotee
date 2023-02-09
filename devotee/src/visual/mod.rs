use crate::util::getter::Getter;
use crate::util::vector::Vector;
use color::Color;

/// Image with dimensions unknown at compile-time.
pub mod canvas;
/// Color system used in `devotee`.
pub mod color;
/// Image with compile-time known dimensions.
pub mod sprite;

mod generalization;

/// Collection of drawing traits and functions compiles in a single prelude.
pub mod prelude {
    pub use super::color::Color;
    pub use super::{draw, paint, printer, stamp};
    pub use super::{Circle, Draw, Image, Line, Pixel, PixelMod, Rect, Text, Triangle};
}

/// Mapper function accepts `x` and `y` coordinates and pixel value.
pub type Mapper<P> = dyn FnMut(i32, i32, P) -> P;

/// Helper paint function for pixel value override.
/// It ignores the value of original pixel and replaces it with `value`.
pub fn paint<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone,
{
    move |_, _, _| value.clone()
}

/// Helper draw function for pixel value combining.
/// It mixes original pixel value and provided `value`.
pub fn draw<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone + Color,
{
    move |_, _, pixel| Color::mix(pixel, value.clone())
}

/// Helper printer mapper for the `Text` trait.
/// It breaks lines on newline symbol (`'\n'`) and ignores any special characters.
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

/// Helper stamper mapper for image-to-image mapping.
/// It just mixes original pixels and pixels of stamped image.
pub fn stamp<P>() -> impl FnMut(i32, i32, P, i32, i32, P) -> P
where
    P: Color,
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

/// Allow line drawing.
pub trait Line<I, F>: Draw {
    /// Use provided function on each pixel in a line in inclusive range.
    fn line(&mut self, from: I, to: I, function: F);
}

/// Allow rectangle-related drawing.
pub trait Rect<I, F>: Draw {
    /// Use provided function on each pixel in a rectangle.
    fn filled_rect(&mut self, from: I, to: I, function: F);

    /// Use provided function on each pixel of rectangle bounds.
    fn rect(&mut self, from: I, to: I, function: F);
}

/// Allow triangle-related drawing.
pub trait Triangle<I, F>: Draw {
    /// Use provided function on each pixel in triangle.
    fn filled_triangle(&mut self, vertices: [I; 3], function: F);

    /// Use provided function on each pixel of triangle bounds.
    fn triangle(&mut self, vertex: [I; 3], function: F);
}

/// Allow circle-related drawing.
pub trait Circle<I, F>: Draw {
    /// Use provided function on each pixel in circle.
    fn filled_circle(&mut self, center: I, radius: i32, function: F);

    /// Use provided function on each pixel of circle bounds.
    fn circle(&mut self, center: I, radius: i32, function: F);
}

/// Apply image using provided mapper function.
pub trait Image<I, U, F>: UnsafePixel<I> {
    /// Use provided function and given image on this drawable.
    fn image(&mut self, at: I, image: &U, function: F);
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
