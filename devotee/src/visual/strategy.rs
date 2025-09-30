use std::ops::RangeInclusive;

use crate::util::vector::Vector;
use crate::visual::image::ImageMut;

/// Pixel-related action strategy.
pub enum PixelStrategy<'a, T> {
    /// Overwrite destination pixels with the provided value.
    Overwrite(&'a T),

    /// Apply provided function to calculate the resulting pixel value.
    Modify(&'a mut dyn FnMut((i32, i32), T) -> T),
}

impl<'a, T> PixelStrategy<'a, T>
where
    T: Clone,
{
    pub(super) fn apply(&mut self, position: Vector<i32>, image: &mut dyn ImageMut<Pixel = T>) {
        match self {
            PixelStrategy::Overwrite(value) => image.set_pixel(position, value),
            PixelStrategy::Modify(function) => image.modify_pixel(position, function),
        }
    }

    pub(super) fn apply_to_line(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        skip: i32,
        image: &mut dyn ImageMut<Pixel = T>,
    ) {
        let start = *x.start();
        let end = *x.end();

        let x = if start < end {
            start + skip..=end
        } else {
            end..=start - skip
        };

        match self {
            PixelStrategy::Overwrite(value) => image.set_horizontal_line(x, y, value),
            PixelStrategy::Modify(function) => image.modify_horizontal_line(x, y, function),
        }
    }
}

impl<'a, T> From<&'a T> for PixelStrategy<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Overwrite(value)
    }
}

impl<'a, T, F> From<&'a mut F> for PixelStrategy<'a, T>
where
    F: FnMut((i32, i32), T) -> T,
{
    fn from(value: &'a mut F) -> Self {
        Self::Modify(value)
    }
}
