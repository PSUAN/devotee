use std::ops::RangeInclusive;

use devotee_backend::RenderSurface;

use super::{FastHorizontalWriter, Image};
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
}

impl<P> Image for Canvas<P>
where
    P: Clone,
{
    type Pixel = P;
    type PixelRef<'a> = &'a P where P:'a;
    type PixelMut<'a> = &'a mut P where P:'a;
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

    /// Get reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> &P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[x + self.width * y]
    }

    /// Get mutable reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> &mut P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[x + self.width * y]
    }

    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }

    fn clear(&mut self, color: P) {
        self.data = vec![color; self.width * self.height].into_boxed_slice();
    }

    fn fast_horizontal_writer(&mut self) -> Option<impl FastHorizontalWriter<Self>> {
        Some(CanvasFastHorizontalWriter { canvas: self })
    }
}

impl<P> RenderSurface for Canvas<P>
where
    P: Clone,
{
    type Data = P;

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn data(&self, x: usize, y: usize) -> P {
        unsafe { self.unsafe_pixel(Vector::new(x as i32, y as i32)).clone() }
    }
}

struct CanvasFastHorizontalWriter<'a, P> {
    canvas: &'a mut Canvas<P>,
}

impl<'a, P> FastHorizontalWriter<Canvas<P>> for CanvasFastHorizontalWriter<'a, P>
where
    P: Clone,
{
    fn write_line<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        function: &mut F,
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
                *pixel = function(x, y, pixel.clone());
            });
    }
}
