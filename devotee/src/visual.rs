use std::ops::{DerefMut, RangeInclusive};

use image::{DesignatorMut, Image, ImageMut};
use strategy::PixelStrategy;

use crate::util::vector::Vector;
use crate::visual::image::{PixelMut, PixelRef};

/// General image-related traits.
pub mod image;

/// Image with dimensions unknown at compile-time.
pub mod canvas;
/// Image with compile-time known dimensions.
pub mod sprite;

/// A view into some image.
pub mod view;

/// Pixel-perfect operations implementation.
pub mod pixel;
/// Subpixel-perfect operations implementation.
pub mod subpixel;

/// Surface adapter implementation.
pub mod adapter;

/// Drawing strategy definitions.
pub mod strategy;

mod util;

/// Helper printer mapper for the `Text` trait.
/// It breaks lines on newline symbol (`'\n'`) and ignores any special characters.
pub fn printer<U>() -> impl FnMut(char, &U) -> Vector<i32>
where
    U: Image,
{
    let mut column = 0;
    let mut line = 0;
    move |code_point, representation| {
        let result = (column, line).into();
        if code_point == '\n' {
            line += representation.height();
            column = 0;
        } else {
            column += representation.width();
        }
        result
    }
}

#[derive(Clone, Copy, Debug)]
enum Scan<T> {
    None,
    Single(T),
    Inclusive(T, T),
}

impl<T> Scan<T> {
    fn rev(self) -> Self {
        match self {
            Scan::None => Scan::None,
            Scan::Single(a) => Scan::Single(a),
            Scan::Inclusive(a, b) => Scan::Inclusive(b, a),
        }
    }

    fn start_unchecked(self) -> T {
        match self {
            Scan::None => unimplemented!("There is no start value for Scan with None variant"),
            Scan::Single(a) => a,
            Scan::Inclusive(a, _) => a,
        }
    }

    fn end_unchecked(self) -> T {
        match self {
            Scan::None => unimplemented!("There is no end value for Scan with None variant"),
            Scan::Single(a) => a,
            Scan::Inclusive(_, b) => b,
        }
    }
}

impl<T> Scan<T>
where
    T: Ord,
{
    fn sorted(self) -> Self {
        if let Scan::Inclusive(a, b) = self {
            if a > b {
                Scan::Inclusive(b, a)
            } else {
                Scan::Inclusive(a, b)
            }
        } else {
            self
        }
    }
}

impl IntoIterator for Scan<i32> {
    type Item = i32;
    type IntoIter = ScanIterator<i32>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Scan::None => ScanIterator {
                current: 0,
                scan: self,
                exhausted: true,
            },
            Scan::Single(a) => ScanIterator {
                current: a,
                scan: self,
                exhausted: false,
            },
            Scan::Inclusive(a, b) if a == b => ScanIterator {
                current: a,
                scan: Scan::Single(a),
                exhausted: false,
            },
            Scan::Inclusive(a, _) => ScanIterator {
                current: a,
                scan: self,
                exhausted: false,
            },
        }
    }
}

struct ScanIterator<T> {
    current: T,
    scan: Scan<T>,
    exhausted: bool,
}

impl Iterator for ScanIterator<i32> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            None
        } else {
            let result = self.current;
            match self.scan {
                Scan::None => unreachable!(),
                Scan::Single(_) => {
                    self.exhausted = true;
                }
                Scan::Inclusive(a, b) => {
                    self.current += (b - a).signum();
                    self.exhausted = result == b;
                }
            }
            Some(result)
        }
    }
}

/// Painter to draw on encapsulated target.
pub struct Painter<'image, I> {
    target: &'image mut I,
    offset: Vector<i32>,
}

impl<'image, I> Painter<'image, I> {
    /// Create new painter instance.
    pub fn new(target: &'image mut I) -> Self {
        let offset = Vector::<i32>::zero();
        Self { target, offset }
    }

    /// Get offset of this `Painter`.
    pub fn offset(&self) -> Vector<i32> {
        self.offset
    }

    /// Get mutable reference to the offset of this `Painter`.
    pub fn offset_mut(&mut self) -> &mut Vector<i32> {
        &mut self.offset
    }

    /// Consume this `Painter` and produce a new one with desired offset.
    pub fn with_offset(self, offset: Vector<i32>) -> Self {
        Self { offset, ..self }
    }
}

impl<T> Painter<'_, T>
where
    T: Image,
{
    fn position_i32(&self, original: Vector<i32>) -> Vector<i32> {
        original + self.offset
    }

    fn position_f32(&self, original: Vector<f32>) -> Vector<f32> {
        original + self.offset.map(|v| v as _)
    }

    /// Get target's width.
    pub fn width(&self) -> i32 {
        Image::width(self.target)
    }

    /// Get target's height.
    pub fn height(&self) -> i32 {
        Image::height(self.target)
    }
}

impl<T> Painter<'_, T>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn apply_strategy(&mut self, position: Vector<i32>, strategy: &mut PixelStrategy<T>) {
        if let Some(mut pixel) = ImageMut::pixel_mut(self.target, position) {
            strategy.apply(position.split(), pixel.deref_mut());
        }
    }

    /// Clear the target with provided color.
    pub fn clear(&mut self, clear_color: T::Pixel) {
        ImageMut::clear(self.target, clear_color)
    }

    fn vertical_line(
        &mut self,
        x: i32,
        from_y: i32,
        to_y: i32,
        strategy: &mut PixelStrategy<T>,
        skip: usize,
    ) {
        let mut iter = from_y..=to_y;
        let mut iter_rev = (to_y..=from_y).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_y < to_y {
            &mut iter
        } else {
            &mut iter_rev
        };

        for y in iter_ref.skip(skip) {
            let pose = (x, y);
            self.apply_strategy(pose.into(), strategy);
        }
    }

    fn horizontal_line(
        &mut self,
        from_x: i32,
        to_x: i32,
        y: i32,
        strategy: &mut PixelStrategy<T>,
        skip: usize,
    ) {
        if let Some(mut writer) = self.target.fast_horizontal_writer() {
            let skip = skip as i32;
            let range = if from_x < to_x {
                (from_x + skip).min(to_x)..=to_x
            } else {
                from_x..=(to_x - skip).max(from_x)
            };
            match strategy {
                PixelStrategy::Overwrite(value) => writer.overwrite(range, y, value),
                PixelStrategy::Modify(modifier) => writer.apply_function(range, y, modifier),
            }
        } else {
            self.slow_horizontal_line(from_x, to_x, y, strategy, skip);
        }
    }

    fn slow_horizontal_line(
        &mut self,
        from_x: i32,
        to_x: i32,
        y: i32,
        strategy: &mut PixelStrategy<T>,
        skip: usize,
    ) {
        let mut iter = from_x..=to_x;
        let mut iter_rev = (to_x..=from_x).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_x < to_x {
            &mut iter
        } else {
            &mut iter_rev
        };

        for x in iter_ref.skip(skip) {
            let position = (x, y);
            if let Some(mut pixel) = ImageMut::pixel_mut(self.target, position.into()) {
                strategy.apply(position, pixel.deref_mut());
            }
        }
    }

    fn filled_rect(&mut self, from: Vector<i32>, to: Vector<i32>, strategy: &mut PixelStrategy<T>) {
        for y in from.y()..=to.y() {
            self.horizontal_line(from.x(), to.x(), y, strategy, 0);
        }
    }
}

/// Painter trait to generalize pixel-perfect and subpixel paint routines.
pub trait Paint<T, C>
where
    T: ImageMut,
{
    /// Get reference to pixel.
    fn pixel(&self, position: Vector<C>) -> Option<PixelRef<'_, T>>;

    /// Get mutable reference to pixel.
    fn pixel_mut(&mut self, position: Vector<C>) -> Option<PixelMut<'_, T>>;

    /// Use passed strategy on a pixel at the given position.
    fn mod_pixel<S>(&mut self, position: Vector<C>, strategy: S)
    where
        for<'a> S: Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel in the line.
    fn line<'a, S>(&mut self, from: Vector<C>, to: Vector<C>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel in the rectangle.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_f<'a, S>(&mut self, from: Vector<C>, dimensions: Vector<C>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel of rectangle bounds.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_b<'a, S>(&mut self, from: Vector<C>, dimensions: Vector<C>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel in the triangle.
    fn triangle_f<'a, S>(&mut self, vertices: [Vector<C>; 3], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed function on each pixel of the triangle bounds.
    fn triangle_b<'a, S>(&mut self, vertices: [Vector<C>; 3], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel of polygon.
    fn polygon_f<'a, S>(&mut self, vertices: &[Vector<C>], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel of polygon bounds.
    fn polygon_b<'a, S>(&mut self, vertices: &[Vector<C>], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel in the circle.
    fn circle_f<'a, S>(&mut self, center: Vector<C>, radius: C, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;

    /// Use passed strategy on each pixel of the circle bounds.
    fn circle_b<'a, S>(&mut self, center: Vector<C>, radius: C, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>;
}

/// A helper utility for writing horizontal lines faster.
pub trait FastHorizontalWriter<T>
where
    T: ImageMut + ?Sized,
{
    /// Use provided `value` to overwrite every pixel in the given `x` range.
    fn overwrite(&mut self, x: RangeInclusive<i32>, y: i32, value: &T::Pixel);

    /// Apply provided `function` to overwrite every pixel in the given `x` range.
    fn apply_function(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        function: &mut dyn FnMut((i32, i32), T::Pixel) -> T::Pixel,
    );
}
