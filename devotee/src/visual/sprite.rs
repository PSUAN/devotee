use super::{Image, ImageMut};
use crate::util::vector::Vector;

/// Sprite of fixed dimensions.
#[derive(Clone, Copy, Debug)]
pub struct Sprite<P, const W: usize, const H: usize> {
    data: [[P; W]; H],
}

impl<P, const W: usize, const H: usize> Sprite<P, W, H> {
    /// Create new Sprite with given color for each pixel.
    pub const fn with_color(color: P) -> Self
    where
        P: Copy,
    {
        let data = [[color; W]; H];
        Self { data }
    }

    /// Create new Sprite with given data.
    pub const fn from_raw_data(data: [[P; W]; H]) -> Self {
        Self { data }
    }
}

impl<P, const W: usize, const H: usize> Image<P> for Sprite<P, W, H>
where
    P: Clone,
{
    fn pixel(&self, position: Vector<i32>) -> Option<P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= W || y >= H {
            None
        } else {
            Some(self.data[y][x].clone())
        }
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        self.data[y][x].clone()
    }

    fn width(&self) -> i32 {
        W as i32
    }

    fn height(&self) -> i32 {
        H as i32
    }
}

impl<P, const W: usize, const H: usize> ImageMut<P> for Sprite<P, W, H>
where
    P: Copy,
{
    fn set_pixel(&mut self, position: Vector<i32>, value: &P) {
        if position.x() < 0 || position.y() < 0 {
            return;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x < W && y < H {
            self.data[y][x] = *value;
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
        if x < W && y < H {
            self.data[y][x] = function(position.split(), self.data[y][x]);
        }
    }

    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &P) {
        let (x, y) = (position.x() as usize, position.y() as usize);
        self.data[y][x] = *value;
    }

    fn clear(&mut self, color: P) {
        self.data = [[color; W]; H];
    }
}

impl<P, const W: usize, const H: usize> Default for Sprite<P, W, H>
where
    P: Default + Copy,
{
    fn default() -> Self {
        Self::with_color(Default::default())
    }
}
