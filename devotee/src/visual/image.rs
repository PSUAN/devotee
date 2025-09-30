use std::ops::RangeInclusive;

use crate::util::vector::Vector;

/// General image trait.
pub trait Image {
    /// Internal pixel representation.
    type Pixel;

    /// Get specific pixel.
    fn pixel(&self, position: Vector<i32>) -> Option<Self::Pixel>;

    /// Get specific pixel without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), (width - 1, height - 1)]
    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> Self::Pixel;

    /// Get width of this image.
    fn width(&self) -> i32;

    /// Get height of this image.
    fn height(&self) -> i32;
}

/// Mutable part of an Image.
pub trait ImageMut: Image {
    /// Set specific pixel as `position`.
    fn set_pixel(&mut self, position: Vector<i32>, value: &Self::Pixel);

    /// Modify specific pixel using provided function.
    fn modify_pixel(
        &mut self,
        position: Vector<i32>,
        function: &mut dyn FnMut((i32, i32), Self::Pixel) -> Self::Pixel,
    );

    /// Set horizontal line values.
    fn set_horizontal_line(&mut self, x_range: RangeInclusive<i32>, y: i32, value: &Self::Pixel) {
        for x in x_range {
            self.set_pixel(Vector::new(x, y), value);
        }
    }

    /// Modify all pixels in the horizontal line.
    fn modify_horizontal_line(
        &mut self,
        x_range: RangeInclusive<i32>,
        y: i32,
        function: &mut dyn FnMut((i32, i32), Self::Pixel) -> Self::Pixel,
    ) {
        for x in x_range {
            self.modify_pixel(Vector::new(x, y), function);
        }
    }

    /// Set specific pixel value without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), (width - 1, height - 1)]
    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &Self::Pixel);

    /// Clear this image with color provided.
    fn clear(&mut self, color: Self::Pixel);
}
