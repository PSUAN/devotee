use std::ops::{Deref, DerefMut, RangeInclusive};

use image::{DesignatorMut, DesignatorRef, Image, ImageMut, PixelMut, PixelRef};

use crate::util::vector::Vector;

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

mod util;

/// Collection of drawing traits and functions in a single prelude.
pub mod prelude {
    pub use super::image::{Image, ImageMut};
    pub use super::view::View;
    pub use super::Paint;
    pub use super::{paint, printer, stamp};
    pub use super::{PaintTarget, Painter};
}

/// Mapper function accepts `x` and `y` coordinates and pixel value.
pub type Mapper<P> = dyn FnMut(i32, i32, P) -> P;

/// Helper paint function for pixel value override.
/// It ignores the value of original pixel and replaces it with `value`.
pub fn paint<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone,
{
    move |_, _, _| value.clone()
}

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

/// Helper stamper mapper for image-to-image mapping.
/// It stamps pixels of the drawn image ignoring values of the original.
pub fn stamp<P>() -> impl FnMut(i32, i32, P, i32, i32, P) -> P {
    move |_, _, _original, _, _, other| other
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

/// Something that can be painted on.
pub trait PaintTarget<T> {
    /// Get painter for painting.
    fn painter<C>(&mut self) -> Painter<T, C>
    where
        C: Clone + Default;
}

impl<T> PaintTarget<T> for T {
    fn painter<C>(&mut self) -> Painter<T, C>
    where
        C: Clone + Default,
    {
        Painter::new(self)
    }
}

/// Painter to draw on encapsulated target.
pub struct Painter<'image, I, C> {
    target: &'image mut I,
    offset: Vector<C>,
}

impl<'image, I, C> Painter<'image, I, C>
where
    C: Clone + Default,
{
    fn new(target: &'image mut I) -> Self {
        Self {
            target,
            offset: Default::default(),
        }
    }

    /// Get new painter with desired offset.
    pub fn with_offset(self, offset: Vector<C>) -> Self {
        Self { offset, ..self }
    }

    /// Set offset for this particular painter.
    pub fn set_offset(&mut self, offset: Vector<C>) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Get offset of this painter.
    pub fn offset(&self) -> Vector<C> {
        self.offset.clone()
    }

    /// Get mutable reference to offset in this painter.
    pub fn offset_mut(&mut self) -> &mut Vector<C> {
        &mut self.offset
    }
}

impl<T, C> Painter<'_, T, C>
where
    T: ImageMut,
    T::Pixel: Clone,
{
    /// Get target's width.
    pub fn width(&self) -> i32 {
        Image::width(self.target)
    }

    /// Get target's height.
    pub fn height(&self) -> i32 {
        Image::height(self.target)
    }

    /// Clear the target with provided color.
    pub fn clear(&mut self, clear_color: T::Pixel) {
        ImageMut::clear(self.target, clear_color)
    }

    fn map_on_pixel_raw<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        point: Vector<i32>,
        function: &mut F,
    ) where
        for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
    {
        if let Some(mut pixel) = self.target.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_vertical_line_raw<'this, 'pixel, F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &'this mut self,
        x: i32,
        from_y: i32,
        to_y: i32,
        function: &mut F,
        skip: usize,
    ) where
        for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
    {
        if x < 0 || x >= self.target.width() {
            return;
        }

        let mut iter = from_y..=to_y;
        let mut iter_rev = (to_y..=from_y).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_y < to_y {
            &mut iter
        } else {
            &mut iter_rev
        };

        for y in iter_ref.skip(skip) {
            let pose = (x, y);
            self.map_on_pixel_raw(pose.into(), function);
        }
    }

    fn map_fast_horizontal_line_raw<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        from_x: i32,
        to_x: i32,
        y: i32,
        function: &mut F,
    ) where
        for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
    {
        if self
            .target
            .fast_horizontal_writer()
            .map(|mut fast| fast.write_line(from_x..=to_x, y, function))
            .is_none()
        {
            self.map_horizontal_line_raw(from_x, to_x, y, function, 0);
        }
    }

    fn map_horizontal_line_raw<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        from_x: i32,
        to_x: i32,
        y: i32,
        function: &mut F,
        skip: usize,
    ) where
        for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
    {
        if y < 0 || y >= self.target.height() {
            return;
        }

        let mut iter = from_x..=to_x;
        let mut iter_rev = (to_x..=from_x).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_x < to_x {
            &mut iter
        } else {
            &mut iter_rev
        };

        for x in iter_ref.skip(skip) {
            let pose = (x, y);
            self.map_on_pixel_raw(pose.into(), function);
        }
    }

    fn map_on_filled_rect_raw<'this, F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &'this mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) where
        for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
        for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
    {
        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = (to.x()).min(self.target.width());
        let end_y = (to.y()).min(self.target.height());

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y).into();
                // SAFETY: we believe that start and end values are in proper bounds.
                unsafe {
                    let pixel = function(x, y, self.target.unsafe_pixel(step).clone());
                    *self.target.unsafe_pixel_mut(step) = pixel;
                }
            }
        }
    }
}

/// Painter trait to generalize pixel-perfect and subpixel paint routines.
pub trait Paint<T, C, I>
where
    T: ImageMut,
    I: Into<Vector<C>>,
{
    /// Get reference to pixel.
    fn pixel(&self, position: I) -> Option<PixelRef<'_, T>>;

    /// Get mutable reference to pixel.
    fn pixel_mut(&mut self, position: I) -> Option<PixelMut<'_, T>>;

    /// Use passed function on a pixel at the given position.
    fn mod_pixel<F>(&mut self, position: I, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel in line.
    fn line<F>(&mut self, from: I, to: I, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel in filled rectangle.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_f<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel of rectangle bounds.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_b<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel in triangle.
    fn triangle_f<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel of triangle bounds.
    fn triangle_b<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel of polygon.
    fn polygon_f<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel of polygon bounds.
    fn polygon_b<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel in circle.
    fn circle_f<F>(&mut self, center: I, radius: C, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;

    /// Use passed function on each pixel of circle bounds.
    fn circle_b<F>(&mut self, center: I, radius: C, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel;
}

/// A helper utility for writing horizontal lines faster.
pub trait FastHorizontalWriter<I>
where
    I: ImageMut + ?Sized,
{
    /// Apply provided function to all pixels in a horizontal line.
    fn write_line<F: FnMut(i32, i32, I::Pixel) -> I::Pixel>(
        &mut self,
        x: RangeInclusive<i32>,
        y: i32,
        function: &mut F,
    );
}
