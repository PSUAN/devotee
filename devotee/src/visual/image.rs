use std::ops::RangeInclusive;

use crate::util::vector::Vector;

use super::view::View;
use super::FastHorizontalWriter;

/// Type trait that can point on a pixel reference.
pub trait DesignatorRef<'a, _Bound = &'a Self> {
    /// Pixel reference.
    type PixelRef;
}

/// Type trait that can point on a mutable pixel reference.
pub trait DesignatorMut<'a, _Bound = &'a mut Self> {
    /// Mutable pixel reference.
    type PixelMut;
}

/// Helper type to represent pixel reference.
pub type PixelRef<'a, This> = <This as DesignatorRef<'a>>::PixelRef;

/// Helper type to represent mutable pixel reference.
pub type PixelMut<'a, This> = <This as DesignatorMut<'a>>::PixelMut;

/// General image trait.
pub trait Image: for<'a> DesignatorRef<'a> {
    /// Pixel type of this image.
    type Pixel;

    /// Get specific pixel reference.
    fn pixel(&self, position: Vector<i32>) -> Option<PixelRef<'_, Self>>;

    /// Get specific pixel reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), (width - 1, height - 1)]
    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> PixelRef<'_, Self>;

    /// Get width of this image.
    fn width(&self) -> i32;

    /// Get height of this image.
    fn height(&self) -> i32;

    /// Get dimensions of this image.
    fn dimensions(&self) -> Vector<i32> {
        Vector::new(self.width(), self.height())
    }

    /// Get an immutable view into this `Image`.
    /// Resulting `View`'s origin and dimensions are cropped to the image automatically.
    fn view(&self, origin: Vector<i32>, dimensions: Vector<i32>) -> View<&Self> {
        View::<&Self>::new(self, origin, dimensions)
    }
}

/// Mutable part of an Image.
pub trait ImageMut: Image + for<'a> DesignatorMut<'a> {
    /// Get specific pixel mutable reference.
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<PixelMut<'_, Self>>;

    /// Get specific pixel mutable reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), (width - 1, height - 1)]
    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> PixelMut<'_, Self>;

    /// Clear this image with color provided.
    fn clear(&mut self, color: Self::Pixel);

    /// Get optional `FastHorizontalWriter` for faster horizontal line drawing.
    fn fast_horizontal_writer(&mut self) -> Option<impl FastHorizontalWriter<Self>> {
        None::<FastHorizontalWriterPlaceholder>
    }

    /// Get a mutable view into this `Image`.
    /// Resulting `View`'s origin and dimensions are cropped to the image automatically.
    fn view_mut<'this>(
        &'this mut self,
        origin: Vector<i32>,
        dimensions: Vector<i32>,
    ) -> View<&'this mut Self> {
        View::<&'this mut Self>::new(self, origin, dimensions)
    }
}

struct FastHorizontalWriterPlaceholder;

impl<I> FastHorizontalWriter<I> for FastHorizontalWriterPlaceholder
where
    I: ImageMut + ?Sized,
{
    fn write_line<F: FnMut(i32, i32, I::Pixel) -> I::Pixel>(
        &mut self,
        _: RangeInclusive<i32>,
        _: i32,
        _: &mut F,
    ) {
        unreachable!()
    }
}
