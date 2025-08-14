use std::ops::RangeInclusive;

use super::image::{DesignatorMut, DesignatorRef};
use super::{FastHorizontalWriter, Image, ImageMut};
use crate::util::vector::Vector;

/// Canvas based on box slice of pixel data.
/// The canvas size is not known at compile time.
#[derive(Clone, Debug)]
pub struct Canvas<P> {
    data: Box<[P]>,
    width: usize,
    height: usize,
}

impl<P> Canvas<P>
where
    P: Clone,
{
    /// Create new canvas with given color and resolution.
    pub fn with_resolution(color: P, width: usize, height: usize) -> Self {
        let data = vec![color; width * height].into_boxed_slice();
        Self {
            data,
            width,
            height,
        }
    }

    /// Get raw internal data as a continuous slice.
    pub fn raw_data(&self) -> &[P] {
        &self.data
    }
}

impl<'a, P> DesignatorRef<'a> for Canvas<P> {
    type PixelRef = &'a P;
}

impl<P> Image for Canvas<P>
where
    P: Clone,
{
    type Pixel = P;

    fn pixel(&self, position: Vector<i32>) -> Option<&P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data.get(x + self.width * y)
        }
    }

    /// Get reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> &P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[x + self.width * y]
    }

    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }
}

impl<'a, P> DesignatorMut<'a> for Canvas<P> {
    type PixelMut = &'a mut P;
}

impl<P> ImageMut for Canvas<P>
where
    P: Clone,
{
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<&mut P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data.get_mut(x + self.width * y)
        }
    }

    /// Get mutable reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> &mut P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[x + self.width * y]
    }

    fn clear(&mut self, color: P) {
        self.data = vec![color; self.width * self.height].into_boxed_slice();
    }

    fn fast_horizontal_writer(&mut self) -> Option<impl FastHorizontalWriter<Self>> {
        Some(CanvasFastHorizontalWriter { canvas: self })
    }
}

struct CanvasFastHorizontalWriter<'a, P> {
    canvas: &'a mut Canvas<P>,
}

impl<P> FastHorizontalWriter<Canvas<P>> for CanvasFastHorizontalWriter<'_, P>
where
    P: Clone,
{
    fn overwrite(&mut self, x: RangeInclusive<i32>, y: i32, value: &P) {
        if y < 0 || y >= Image::height(self.canvas) {
            return;
        }
        let width = Image::width(self.canvas);
        let start_x = (*x.start()).clamp(0, width - 1);
        let end_x = (*x.end() + 1).clamp(0, width - 1);
        let start = start_x + width * y;
        let end = end_x + width * y;

        let s = start.min(end) as usize;
        let e = start.max(end) as usize;

        self.canvas.data[s..e].fill(value.clone());
    }

    fn apply_function(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        function: &mut dyn FnMut((i32, i32), P) -> P,
    ) {
        if y < 0 || y >= Image::height(self.canvas) {
            return;
        }
        let width = Image::width(self.canvas);
        let start_x = (*x.start()).clamp(0, width - 1);
        let end_x = (*x.end() + 1).clamp(0, width - 1);
        let start = start_x + width * y;
        let end = end_x + width * y;

        let s = start.min(end) as usize;
        let e = start.max(end) as usize;

        self.canvas.data[s..e]
            .iter_mut()
            .enumerate()
            .for_each(|(x, pixel)| {
                let x = start_x + x as i32;
                *pixel = function((x, y), pixel.clone());
            });
    }
}
