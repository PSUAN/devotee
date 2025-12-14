use super::{Image, ImageMut};
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

impl<P> Image<P> for Canvas<P>
where
    P: Clone,
{
    fn pixel(&self, position: Vector<i32>) -> Option<P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= self.width || y >= self.height {
            None
        } else {
            self.data.get(x + self.width * y).cloned()
        }
    }

    /// Get reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        self.data[x + self.width * y].clone()
    }

    fn width(&self) -> i32 {
        self.width as i32
    }

    fn height(&self) -> i32 {
        self.height as i32
    }
}

impl<P> ImageMut<P> for Canvas<P>
where
    P: Clone,
{
    fn set_pixel(&mut self, position: Vector<i32>, value: &P) {
        if position.x() < 0 || position.y() < 0 {
            return;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x < self.width && y < self.height {
            self.data[x + self.width * y] = value.clone();
        }
    }

    fn modify_pixel(
        &mut self,
        position: Vector<i32>,
        function: &mut dyn FnMut((i32, i32), P) -> P,
    ) {
        if position.x() < 0 || position.y() < 0 {
            return;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x < self.width && y < self.height {
            self.data[x + self.width * y] =
                function(position.split(), self.data[x + self.width * y].clone());
        }
    }

    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &P) {
        let (x, y) = (position.x() as usize, position.y() as usize);
        self.data[x + self.width * y] = value.clone();
    }

    fn clear(&mut self, color: P) {
        self.data.fill(color);
    }
}
