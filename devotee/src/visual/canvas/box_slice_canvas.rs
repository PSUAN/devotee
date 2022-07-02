use super::Canvas;
use crate::math::vector::Vector;
use std::slice::Iter;

/// Canvas based on box slice of pixel data.
pub struct BoxSliceCanvas<P> {
    data: Box<[P]>,
    width: usize,
    height: usize,
}

impl<P> BoxSliceCanvas<P>
where
    P: Copy,
{
    /// Create new canvas with given resolution.
    pub fn with_resolution(color: P, width: usize, height: usize) -> Self {
        let data = vec![color; width * height].into_boxed_slice();
        Self {
            data,
            width,
            height,
        }
    }
}

impl<P> BoxSliceCanvas<P> {
    /// Get canvas width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get canvas height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get individual canvas pixel.
    pub fn pixel(&self, x: usize, y: usize) -> Option<&P> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data.get(x + self.width * y)
        }
    }

    /// Get mutable reference to pixel.
    pub fn pixel_mut(&mut self, x: usize, y: usize) -> Option<&mut P> {
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data.get_mut(x + self.width * y)
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, P> {
        self.data.iter()
    }
}

impl<P> Canvas for BoxSliceCanvas<P>
where
    P: Copy,
{
    type Index = Vector<i32>;
    type Pixel = P;

    fn pixel(&self, index: Self::Index) -> Option<&Self::Pixel> {
        if index.x() < 0 || index.y() < 0 {
            None
        } else {
            BoxSliceCanvas::pixel(self, index.x() as usize, index.y() as usize)
        }
    }

    fn pixel_mut(&mut self, index: Self::Index) -> Option<&mut Self::Pixel> {
        if index.x() < 0 || index.y() < 0 {
            None
        } else {
            BoxSliceCanvas::pixel_mut(self, index.x() as usize, index.y() as usize)
        }
    }

    fn resolution(&self) -> Self::Index {
        (self.width as i32, self.height as i32).into()
    }

    fn clear(&mut self, color: Self::Pixel) {
        self.data = vec![color; self.width * self.height].into_boxed_slice();
    }
}
