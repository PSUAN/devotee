use super::color::Color;
use index_iteration::{BoxIteration, LineIteration};
use std::ops::Add;

/// Two-dimensional canvas based on box slice.
pub mod box_slice_canvas;
/// Various interpolation algorithms.
pub mod index_iteration;

/// Generalized canvas trait.
pub trait Canvas {
    /// Pixel indexing type.
    type Index;
    /// Specific pixel type.
    type Pixel;

    /// Get specific canvas pixel reference based on index.
    fn pixel(&self, index: Self::Index) -> Option<&Self::Pixel>;
    /// Get specific canvas pixel mutable reference based on index.
    fn pixel_mut(&mut self, index: Self::Index) -> Option<&mut Self::Pixel>;
    /// Get canvas resolution.
    fn resolution(&self) -> Self::Index;
    /// Clear canvas with given color.
    fn clear(&mut self, pixel: Self::Pixel);
}

/// Canvas trait extension for pixel setting.
pub trait CanvasSet: Canvas
where
    Self::Pixel: Copy,
    Self::Index: Copy + Default + LineIteration + BoxIteration + Add<Output = Self::Index>,
{
    /// Try setting specific canvas pixel.
    /// Returns value of pixel set.
    fn set_pixel<P: Into<Self::Pixel>, I: Into<Self::Index>>(
        &mut self,
        index: I,
        pixel: P,
    ) -> Option<Self::Pixel> {
        if let Some(value) = self.pixel_mut(index.into()) {
            *value = pixel.into();
            Some(*value)
        } else {
            None
        }
    }

    /// Sets pixel line between `from` and `to`.
    fn set_line<P: Into<Self::Pixel>, I: Into<Self::Index>>(&mut self, from: I, to: I, pixel: P) {
        let iterator = LineIteration::iterator(from.into(), to.into());
        let pixel = pixel.into();
        for pose in iterator {
            self.set_pixel(pose, pixel);
        }
    }

    /// Sets pixels in given rectangle.
    fn set_filled_rect<P: Into<Self::Pixel>, I: Into<Self::Index>>(
        &mut self,
        from: I,
        to: I,
        pixel: P,
    ) {
        let iterator = BoxIteration::iterator(from.into(), to.into());
        let pixel = pixel.into();
        for pose in iterator {
            self.set_pixel(pose, pixel);
        }
    }

    /// Sets pixels to values specified in `image`.
    fn set_image<I: Into<Self::Index>>(
        &mut self,
        image: &dyn Canvas<Index = Self::Index, Pixel = Self::Pixel>,
        at: I,
    ) {
        let at = at.into();
        let start = Self::Index::default();
        let iterator = BoxIteration::iterator(start, image.resolution());
        for pose in iterator {
            if let Some(pixel) = image.pixel(start + pose) {
                self.set_pixel(at + pose, *pixel);
            }
        }
    }
}

/// Canvas extension for pixel drawing.
pub trait CanvasDraw: Canvas
where
    Self::Pixel: Color + Copy,
    Self::Index: Copy + Default + LineIteration + BoxIteration + Add<Output = Self::Index>,
{
    /// Try drawing over specific canvas pixel.
    /// Returns value of resulting pixel.
    fn draw_pixel(&mut self, index: Self::Index, pixel: Self::Pixel) -> Option<Self::Pixel> {
        if let Some(value) = self.pixel_mut(index) {
            *value = value.mix(pixel);
            Some(*value)
        } else {
            None
        }
    }

    /// Draw pixel line between `from` and `to`.
    fn draw_line<P: Into<Self::Pixel>, I: Into<Self::Index>>(&mut self, from: I, to: I, pixel: P) {
        let iterator = LineIteration::iterator(from.into(), to.into());
        let pixel = pixel.into();
        for pose in iterator {
            self.draw_pixel(pose, pixel);
        }
    }

    /// Draw pixels in given rectangle.
    fn draw_filled_rect<P: Into<Self::Pixel>, I: Into<Self::Index>>(
        &mut self,
        from: I,
        to: I,
        pixel: P,
    ) {
        let iterator = BoxIteration::iterator(from.into(), to.into());
        let pixel = pixel.into();
        for pose in iterator {
            self.draw_pixel(pose, pixel);
        }
    }

    /// Draw pixels to values specified in `image`.
    fn draw_image<I: Into<Self::Index>>(
        &mut self,
        image: &dyn Canvas<Index = Self::Index, Pixel = Self::Pixel>,
        at: I,
    ) {
        let at = at.into();
        let start = Self::Index::default();
        let iterator = BoxIteration::iterator(start, image.resolution());
        for pose in iterator {
            if let Some(pixel) = image.pixel(start + pose) {
                self.draw_pixel(at + pose, *pixel);
            }
        }
    }
}

impl<T> CanvasSet for T
where
    T: Canvas,
    T::Pixel: Copy,
    T::Index: Copy + Default + LineIteration + BoxIteration + Add<Output = T::Index>,
{
}

impl<T> CanvasDraw for T
where
    T: Canvas,
    T::Pixel: Color + Copy,
    T::Index: Copy + Default + LineIteration + BoxIteration + Add<Output = T::Index>,
{
}
