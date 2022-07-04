use super::color::Color;
use crate::math::vector::Vector;
use std::mem;
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

    /// Apply given function to each pixel at a line from the `from` position to the `to` position.
    pub fn map_on_line<I: Into<Vector<i32>>, F: FnMut(P) -> P>(
        &mut self,
        from: I,
        to: I,
        function: F,
    ) {
        let mut function = function;
        let mut from = from.into();
        let mut to = to.into();
        if from.x() == to.x() {
            self.map_vertical_line(from.x(), from.y(), to.y(), function);
            return;
        }
        if from.y() == to.y() {
            self.map_horizontal_line(from.x(), to.x(), from.y(), function);
            return;
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

            if pose.x() >= 0
                && pose.y() >= 0
                && pose.x() < self.width as i32
                && pose.y() < self.height as i32
            {
                unsafe {
                    let pixel = function(self.pixel_unsafe(pose).clone());
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }

    /// Apply given function to each pixel in a given rectangle.
    pub fn map_on_filled_rect<I: Into<Vector<i32>>, F: FnMut(P) -> P>(
        &mut self,
        from: I,
        to: I,
        function: F,
    ) {
        let mut function = function;
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
                    let pixel = function(self.pixel_unsafe(step).clone());
                    *self.pixel_mut_unsafe(step) = pixel;
                }
            }
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, P> {
        self.data.iter()
    }

    /// Get reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    pub unsafe fn pixel_unsafe<I: Into<Vector<i32>>>(&self, position: I) -> &P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[x + self.width * y]
    }

    /// Get mutable reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    pub unsafe fn pixel_mut_unsafe<I: Into<Vector<i32>>>(&mut self, position: I) -> &mut P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[x + self.width * y]
    }

    fn map_vertical_line<F: FnMut(P) -> P>(
        &mut self,
        x: i32,
        mut from_y: i32,
        mut to_y: i32,
        mut function: F,
    ) {
        if x < 0 || x >= self.width as i32 {
            return;
        }
        if from_y > to_y {
            mem::swap(&mut from_y, &mut to_y);
        }
        from_y = from_y.max(0);
        to_y = to_y.min(self.height as i32);
        for y in from_y..to_y {
            let step = (x, y);
            unsafe {
                let pixel = function(self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_horizontal_line<F: FnMut(P) -> P>(
        &mut self,
        mut from_x: i32,
        mut to_x: i32,
        y: i32,
        mut function: F,
    ) {
        if y < 0 || y >= self.height as i32 {
            return;
        }
        if from_x > to_x {
            mem::swap(&mut from_x, &mut to_x);
        }
        from_x = from_x.max(0);
        to_x = to_x.min(self.height as i32);
        for x in from_x..to_x {
            let step = (x, y);
            unsafe {
                let pixel = function(self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    /// Combine two images and apply provided function to result.
    pub fn zip_map_images<I: Into<Vector<i32>>, O: Clone, F: FnMut(P, O) -> P>(
        &mut self,
        at: I,
        image: &Canvas<O>,
        function: F,
    ) {
        let mut function = function;
        let at = at.into();

        let image_start_x = if at.x() < 0 { -at.x() } else { 0 };
        let image_start_y = if at.y() < 0 { -at.y() } else { 0 };

        let image_end_x = if at.x() + image.width as i32 >= self.width as i32 {
            self.width as i32 - at.x()
        } else {
            image.width as i32
        };
        let image_end_y = if at.y() + image.height as i32 >= self.height as i32 {
            self.height as i32 - at.y()
        } else {
            image.height as i32
        };
        for x in image_start_x..image_end_x {
            for y in image_start_y..image_end_y {
                let step = (x, y).into();
                let pose = at + step;
                unsafe {
                    let color = image.pixel_unsafe(step);
                    let pixel = function(self.pixel_unsafe(pose).clone(), color.clone());
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }
}

impl<P> Canvas<P>
where
    P: Clone + Color,
{
    /// Draw a line in given `color` from the `from` position to the `to` position.
    pub fn draw_line<I: Into<Vector<i32>>>(&mut self, from: I, to: I, color: P) {
        self.map_on_line(from, to, |pixel| pixel.mix(color.clone()));
    }

    /// Draw pixel at the given position `at`.
    pub fn draw_pixel<I: Into<Vector<i32>>>(&mut self, at: I, pixel: P) {
        if let Some(value) = self.pixel_mut(at) {
            *value = value.clone().mix(pixel);
        }
    }

    /// Draw filled rect at the given position `from` to the `to` position.
    pub fn draw_filled_rect<I: Into<Vector<i32>>>(&mut self, from: I, to: I, color: P) {
        self.map_on_filled_rect(from, to, |pixel| pixel.mix(color.clone()));
    }

    /// Draw image at the `at` position.
    pub fn draw_image<I: Into<Vector<i32>>>(&mut self, at: I, image: &Canvas<P>) {
        self.zip_map_images(at, image, |pixel, other| pixel.mix(other))
    }
}
