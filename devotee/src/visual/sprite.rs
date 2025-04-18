use std::ops::RangeInclusive;

use super::image::{DesignatorMut, DesignatorRef};
use super::{FastHorizontalWriter, Image, ImageMut};
use crate::util::vector::Vector;

/// Sprite of fixed dimensions.
#[derive(Clone, Copy, Debug)]
pub struct Sprite<P, const W: usize, const H: usize> {
    data: [[P; W]; H],
}

impl<P, const W: usize, const H: usize> Sprite<P, W, H>
where
    P: Copy,
{
    /// Create new Sprite with given color for each pixel.
    pub const fn with_color(color: P) -> Self {
        let data = [[color; W]; H];
        Self { data }
    }

    /// Create new Sprite with given data.
    pub const fn with_data(data: [[P; W]; H]) -> Self {
        Self { data }
    }
}

impl<'a, P, const W: usize, const H: usize> DesignatorRef<'a> for Sprite<P, W, H> {
    type PixelRef = &'a P;
}

impl<P, const W: usize, const H: usize> Image for Sprite<P, W, H> {
    type Pixel = P;

    fn pixel(&self, position: Vector<i32>) -> Option<&P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= W || y >= H {
            None
        } else {
            Some(&self.data[y][x])
        }
    }

    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> &P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[y][x]
    }

    fn width(&self) -> i32 {
        W as i32
    }

    fn height(&self) -> i32 {
        H as i32
    }
}

impl<'a, P, const W: usize, const H: usize> DesignatorMut<'a> for Sprite<P, W, H> {
    type PixelMut = &'a mut P;
}

impl<P, const W: usize, const H: usize> ImageMut for Sprite<P, W, H>
where
    P: Copy,
{
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<&mut P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= W || y >= H {
            None
        } else {
            Some(&mut self.data[y][x])
        }
    }

    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> &mut P {
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[y][x]
    }

    fn clear(&mut self, color: P) {
        self.data = [[color; W]; H];
    }

    fn fast_horizontal_writer(&mut self) -> Option<impl FastHorizontalWriter<Self>> {
        Some(SpriteFastHorizontalWriter { sprite: self })
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

struct SpriteFastHorizontalWriter<'a, P, const W: usize, const H: usize> {
    sprite: &'a mut Sprite<P, W, H>,
}

impl<P, const W: usize, const H: usize> FastHorizontalWriter<Sprite<P, W, H>>
    for SpriteFastHorizontalWriter<'_, P, W, H>
where
    P: Copy,
{
    fn write_line<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        function: &mut F,
    ) {
        if y < 0 || y >= Image::height(self.sprite) {
            return;
        }
        let width = Image::width(self.sprite);
        let start = (*x.start()).clamp(0, width - 1);
        let end = (*x.end() + 1).clamp(0, width - 1);

        let s = start.min(end) as usize;
        let e = start.max(end) as usize;

        self.sprite.data[y as usize][s..e]
            .iter_mut()
            .enumerate()
            .for_each(|(x, pixel)| {
                let x = start + x as i32;
                *pixel = function(x, y, *pixel);
            });
    }
}
