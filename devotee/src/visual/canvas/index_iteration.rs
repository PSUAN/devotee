use crate::math::vector::Vector;
use std::iter::Iterator;
use std::mem;

/// Iterate along line in 2/3 dimensions.
pub trait LineIteration {
    /// Iterator to interpolate over all values in a line.
    type LineIterator: Iterator<Item = Self>;
    /// Generate LineIterator between `self` value and `other` values.
    fn iterator(self, other: Self) -> Self::LineIterator;
}

/// Iterate in a given box/cube/hypercube.
pub trait BoxIteration {
    /// Iterator to iterate over all values in a box.
    type BoxIterator: Iterator<Item = Self>;
    /// Generate BoxIterator between `self` and `other` values.
    fn iterator(self, other: Self) -> Self::BoxIterator;
}

impl LineIteration for Vector<i32> {
    type LineIterator = LineIteratorI32;
    fn iterator(self, other: Self) -> Self::LineIterator {
        LineIteratorI32::new(self, other)
    }
}

/// Line LineIterator for `Vector<i32>`.
pub struct LineIteratorI32 {
    current: Vector<i32>,
    to: Vector<i32>,
    steep: bool,
    positive_y_step: bool,
    error: i32,
    delta_x: i32,
    delta_y: i32,
}

impl LineIteratorI32 {
    fn new(from: Vector<i32>, to: Vector<i32>) -> Self {
        let mut from = from;
        let mut to = to;

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

        let error = delta_x / 2;
        let positive_y_step = from.y() < to.y();

        Self {
            current: from,
            to,
            steep,
            positive_y_step,
            error,
            delta_x,
            delta_y,
        }
    }
}

impl Iterator for LineIteratorI32 {
    type Item = Vector<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.x() <= self.to.x() {
            let result = if self.steep {
                (self.current.y(), self.current.x()).into()
            } else {
                self.current
            };

            self.error -= self.delta_y;
            if self.error < 0 {
                *self.current.y_mut() += if self.positive_y_step { 1 } else { -1 };
                self.error += self.delta_x;
            }

            *self.current.x_mut() += 1;

            Some(result)
        } else {
            None
        }
    }
}

impl BoxIteration for Vector<i32> {
    type BoxIterator = BoxIteratorI32;
    fn iterator(self, other: Self) -> Self::BoxIterator {
        BoxIteratorI32::new(self, other)
    }
}

/// BoxIterator for Vector<i32>.
pub struct BoxIteratorI32 {
    from: Vector<i32>,
    to: Vector<i32>,
    current: Vector<i32>,
}

impl BoxIteratorI32 {
    fn new(from: Vector<i32>, to: Vector<i32>) -> Self {
        let mut from = from;
        let mut to = to;
        if from.x() > to.x() {
            mem::swap(from.x_mut(), to.x_mut());
        }
        if from.y() > to.y() {
            mem::swap(from.y_mut(), to.y_mut());
        }

        let current = from;
        Self { from, to, current }
    }
}

impl Iterator for BoxIteratorI32 {
    type Item = Vector<i32>;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;

        if self.current.y() > self.to.y() {
            return None;
        }

        *self.current.x_mut() += 1;
        if self.current.x() > self.to.x() {
            *self.current.x_mut() = self.from.x();
            *self.current.y_mut() += 1;
        }
        Some(result)
    }
}
