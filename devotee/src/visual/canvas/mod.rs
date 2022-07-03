use super::color::Color;
use crate::math::vector::Vector;
use std::slice::Iter;

/// Canvas based on box slice of pixel data.
pub struct Canvas<P> {
    data: Box<[P]>,
    width: usize,
    height: usize,
}

impl<P> Canvas<P>
where
    P: Clone,
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

impl<P> Canvas<P>
where
    P: Clone,
{
    /// Get canvas width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get canvas height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get individual canvas pixel.
    pub fn pixel<I: Into<Vector<i32>>>(&self, position: I) -> Option<&P> {
        let position = position.into();
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

    /// Get mutable reference to pixel.
    pub fn pixel_mut<I: Into<Vector<i32>>>(&mut self, position: I) -> Option<&mut P> {
        let position = position.into();
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

    /// Clear color buffer with the given color value.
    pub fn clear(&mut self, color: P) {
        self.data = vec![color; self.width * self.height].into_boxed_slice();
    }

    pub(crate) fn iter(&self) -> Iter<'_, P> {
        self.data.iter()
    }

    /// Gen reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn pixel_unsafe<I: Into<Vector<i32>>>(&self, position: I) -> &P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[x + self.width * y]
    }

    /// Gen mutable reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn pixel_mut_unsafe<I: Into<Vector<i32>>>(&mut self, position: I) -> &mut P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[x + self.width * y]
    }
}

impl<P> Canvas<P>
where
    P: Clone + Color,
{
    /// Draw a line in given `color` from the `from` position to the `to` position.
    pub fn draw_line<I: Into<Vector<i32>>>(&mut self, from: I, to: I, color: P) {
        let mut from = from.into();
        let mut to = to.into();
        if from.x() == to.x() {
            // TODO: add optimized `set_vertical_line` here
        }
        if from.y() == to.y() {
            // TODO: add optimized `set_horizontal_line` here
        }

        let steep = (to.y() - from.y()).abs() > (to.x() - from.x()).abs();
        if steep {
            from = (from.y(), from.x()).into();
            to = (to.y(), to.x()).into();
        }

        if from.x() > to.x() {
            (to, from) = ((from.x(), from.y()).into(), (to.x(), to.y()).into());
        }

        let delta_x = to.x() - from.x();
        let delta_y = (to.y() - from.y()).abs();

        let mut error = delta_x / 2;
        let positive_y_step = from.y() < to.y();

        let mut current = from;

        while current.x() <= to.x() {
            let pose = if steep {
                (current.y(), current.x()).into()
            } else {
                current
            };

            error -= delta_y;
            if error < 0 {
                *current.y_mut() += if positive_y_step { 1 } else { -1 };
                error += delta_x;
            }

            *current.x_mut() += 1;

            if pose.x() >= self.width as i32 && pose.y() >= self.height as i32 {
                return;
            }
            if pose.x() > 0 && pose.y() > 0 {
                unsafe {
                    let pixel = self.pixel_unsafe(pose).clone().mix(color.clone());
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }

    /// Draw given image at the given position `at`.
    pub fn draw_image<I: Into<Vector<i32>>>(&mut self, at: I, image: &Canvas<P>) {
        let at = at.into();

        let image_start_x = if at.x() < 0 { -at.x() } else { 0 };
        let image_start_y = if at.y() < 0 { -at.y() } else { 0 };

        let image_end_x = if at.x() + image.width as i32 >= self.width as i32 {
            image.width as i32 - at.x()
        } else {
            image.width as i32
        };
        let image_end_y = if at.y() + image.height as i32 >= self.height as i32 {
            image.height as i32 - at.y()
        } else {
            image.height as i32
        };
        for x in image_start_x..image_end_x {
            for y in image_start_y..image_end_y {
                let step = (x, y).into();
                let pose = at + step;
                unsafe {
                    let color = image.pixel_unsafe(step);
                    let pixel = self.pixel_unsafe(pose).clone().mix(color.clone());
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }

    /// Draw pixel at the given position `at`.
    pub fn draw_pixel<I: Into<Vector<i32>>>(&mut self, at: I, pixel: P) {
        if let Some(value) = self.pixel_mut(at) {
            *value = value.clone().mix(pixel);
        }
    }

    /// Draw filled rect at the given position `at`.
    pub fn draw_filled_rect<I: Into<Vector<i32>>>(&mut self, from: I, to: I, color: P) {
        let from = from.into();
        let to = to.into();

        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = to.x().min(self.width as i32);
        let end_y = to.y().min(self.height as i32);

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y);
                unsafe {
                    let pixel = self.pixel_unsafe(step).clone().mix(color.clone());
                    *self.pixel_mut_unsafe(step) = pixel;
                }
            }
        }
    }
}
