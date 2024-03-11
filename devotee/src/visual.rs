use std::ops::{Deref, DerefMut};

use crate::util::vector::Vector;

/// Image with dimensions unknown at compile-time.
pub mod canvas;
/// Image with compile-time known dimensions.
pub mod sprite;

/// Pixel-perfect operations implementation.
pub mod pixel;
/// Subpixel-perfect operations implementation.
pub mod subpixel;

/// Collection of drawing traits and functions compiles in a single prelude.
pub mod prelude {
    pub use super::{paint, printer, stamp};
    pub use super::{Image, Paint};
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

/// General image trait.
pub trait Image {
    /// Pixel type of this image.
    type Pixel;
    /// Reference to pixel.
    type PixelRef<'a>
    where
        Self: 'a;
    /// Mutable reference to pixel.
    type PixelMut<'a>
    where
        Self: 'a;
    /// Get specific pixel reference.
    fn pixel(&self, position: Vector<i32>) -> Option<Self::PixelRef<'_>>;
    /// Get specific pixel mutable reference.
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<Self::PixelMut<'_>>;
    /// Get specific pixel reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), [width - 1, height - 1]]
    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> Self::PixelRef<'_>;
    /// Get specific pixel mutable reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), [width - 1, height - 1]]
    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> Self::PixelMut<'_>;
    /// Get width of this image.
    fn width(&self) -> i32;
    /// Get height of this image.
    fn height(&self) -> i32;
    /// Clear this image with color provided.
    fn clear(&mut self, color: Self::Pixel);

    /// Get dimensions of this image.
    fn dimensions(&self) -> Vector<i32> {
        Vector::new(self.width(), self.height())
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
pub struct Painter<'a, I, C> {
    target: &'a mut I,
    offset: Vector<C>,
}

impl<'a, I, C> Painter<'a, I, C>
where
    C: Clone + Default,
{
    fn new(target: &'a mut I) -> Self {
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

impl<'a, T, P, C> Painter<'a, T, C>
where
    T: Image<Pixel = P>,
    <T as Image>::Pixel: Clone,
    for<'b> <T as Image>::PixelRef<'b>: Deref<Target = <T as Image>::Pixel>,
    for<'b> <T as Image>::PixelMut<'b>: DerefMut<Target = <T as Image>::Pixel>,
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
    pub fn clear(&mut self, clear_color: P) {
        Image::clear(self.target, clear_color)
    }

    fn map_on_pixel_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        point: Vector<i32>,
        function: &mut F,
    ) {
        if let Some(mut pixel) = self.target.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_vertical_line_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        x: i32,
        from_y: i32,
        to_y: i32,
        function: &mut F,
        skip: usize,
    ) {
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

    fn map_horizontal_line_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        from_x: i32,
        to_x: i32,
        y: i32,
        function: &mut F,
        skip: usize,
    ) {
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

    fn map_on_filled_rect_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) {
        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = (to.x()).min(self.target.width());
        let end_y = (to.y()).min(self.target.height());

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y).into();
                // SAFETY: we start and end values are in proper bounds.
                unsafe {
                    let pixel = function(x, y, self.target.unsafe_pixel(step).clone());
                    *self.target.unsafe_pixel_mut(step) = pixel;
                }
            }
        }
    }
}

/// Painter trait to generalize pixel-perfect and subpixel paint routines.
pub trait Paint<T, C, P, I>
where
    T: Image<Pixel = P>,
    I: Into<Vector<C>>,
{
    /// Get reference to pixel.
    fn pixel(&self, position: I) -> Option<T::PixelRef<'_>>;

    /// Get mutable reference to pixel.
    fn pixel_mut(&mut self, position: I) -> Option<T::PixelMut<'_>>;

    /// Use passed function on a pixel at the given position.
    fn mod_pixel<F>(&mut self, position: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel in line.
    fn line<F>(&mut self, from: I, to: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel in filled rectangle.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_f<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel of rectangle bounds.
    /// The `dimensions` determine size of the rectangle, zero or negative value produces no rectangle.
    fn rect_b<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel in triangle.
    fn triangle_f<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel of triangle bounds.
    fn triangle_b<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel of polygon.
    fn polygon_f<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel of polygon bounds.
    fn polygon_b<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel in circle.
    fn circle_f<F>(&mut self, center: I, radius: C, function: F)
    where
        F: FnMut(i32, i32, P) -> P;

    /// Use passed function on each pixel of circle bounds.
    fn circle_b<F>(&mut self, center: I, radius: C, function: F)
    where
        F: FnMut(i32, i32, P) -> P;
}

/// Pixel iterator provider.
pub trait PixelsIterator<'a, P: 'a> {
    /// Specific iterator to be produced.
    type Iterator: Iterator<Item = &'a P>;

    /// Produce pixels iterator.
    fn pixels(&'a self) -> Self::Iterator;
}

impl<'a, P: 'a, I: PixelsIterator<'a, P> + ?Sized> PixelsIterator<'a, P> for Box<I> {
    type Iterator = I::Iterator;

    fn pixels(&'a self) -> Self::Iterator {
        I::pixels(self)
    }
}
