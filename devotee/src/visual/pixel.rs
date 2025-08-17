use std::ops::{Deref, DerefMut};

use crate::util::getter::Getter;
use crate::util::vector::Vector;
use crate::visual::Paint;

use super::image::{DesignatorMut, DesignatorRef, PixelMut, PixelRef};
use super::strategy::PixelStrategy;
use super::util::AngleIterator;
use super::{Image, ImageMut, Painter, Scan};

type ImageMapper<'a, T, R> = dyn FnMut((i32, i32), T, (i32, i32), R) -> T + 'a;

fn scanline_segment_i32(segment: (Vector<i32>, Vector<i32>), scanline: i32) -> Scan<i32> {
    let (from, to) = if segment.0.y() < segment.1.y() {
        (segment.0, segment.1)
    } else {
        (segment.1, segment.0)
    };

    let (delta_x, delta_y) = (to - from).split();

    if scanline < from.y() || scanline > to.y() {
        return Scan::None;
    }
    if delta_y == 0 {
        return Scan::Inclusive(from.x(), to.x()).sorted();
    }

    let steep = delta_x.abs() < delta_y;
    if steep {
        let y = scanline;
        let x = ((delta_x + 1) * (y - from.y())
            + (delta_x - 1) * (y - to.y())
            + (from.x() + to.x()) * delta_y)
            / (delta_y * 2);

        Scan::Single(x).sorted()
    } else {
        let (left, right) = if from.x() < to.x() {
            (from.x(), to.x())
        } else {
            (to.x(), from.x())
        };

        let y = scanline;
        let center_x = (delta_x + 1) * (y - from.y())
            + (delta_x - 1) * (y - to.y())
            + (from.x() + to.x()) * delta_y;

        let left_x = (delta_x + 1) * (y - from.y() + 1)
            + (delta_x - 1) * (y - to.y() + 1)
            + (from.x() + to.x()) * delta_y;
        let right_x = (delta_x + 1) * (y - from.y() - 1)
            + (delta_x - 1) * (y - to.y() - 1)
            + (from.x() + to.x()) * delta_y;

        let first_x = (center_x + left_x) / (4 * delta_y);
        let second_x = (center_x + right_x) / (4 * delta_y);
        if first_x < second_x {
            Scan::Inclusive(left.max(first_x), right.min(second_x - 1))
        } else {
            Scan::Inclusive(left.max(second_x), right.min(first_x - 1))
        }
    }
}

impl<T> Painter<'_, T>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn paint_line(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        strategy: &mut PixelStrategy<T>,
        skip: usize,
    ) {
        if from.x() == to.x() {
            self.vertical_line(from.x(), from.y(), to.y(), strategy, skip);
            return;
        }
        if from.y() == to.y() {
            self.horizontal_line(from.x(), to.x(), from.y(), strategy, skip);
            return;
        }

        let mut iter = from.y()..=to.y();
        let mut iter_rev = (to.y()..=from.y()).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from.y() < to.y() {
            &mut iter
        } else {
            &mut iter_rev
        };

        let rev = from.x() > to.x();

        let mut skip = skip;

        for y in iter_ref {
            let scan = scanline_segment_i32((from, to), y);
            let mut scan_rev = scan.rev().into_iter();
            let mut scan = scan.into_iter();
            let scan: &mut dyn Iterator<Item = i32> = if rev { &mut scan_rev } else { &mut scan };

            for x in scan {
                if skip == 0 {
                    let pose = (x, y).into();
                    self.apply_strategy(pose, strategy);
                } else {
                    skip -= 1;
                }
            }
        }
    }

    fn filled_triangle(&mut self, vertices: [Vector<i32>; 3], strategy: &mut PixelStrategy<T>) {
        let mut vertices = vertices;
        vertices.sort_by(|a, b| a.y_ref().cmp(b.y_ref()));
        let [a, b, c] = vertices;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertices.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.horizontal_line(
                vertices[0].x(),
                vertices[2].x(),
                vertices[0].y(),
                strategy,
                0,
            );
            return;
        }

        let middle = if b.y() == c.y() { b.y() } else { b.y() - 1 };

        for y in a.y()..=middle {
            let left_range = scanline_segment_i32((a, b), y);
            let right_range = scanline_segment_i32((a, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.horizontal_line(left, right, y, strategy, 0);
        }

        let middle = middle + 1;
        for y in middle..=c.y() {
            let left_range = scanline_segment_i32((a, c), y);
            let right_range = scanline_segment_i32((b, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.horizontal_line(left, right, y, strategy, 0);
        }
    }

    fn filled_polygon_raw(&mut self, vertices: &[Vector<i32>], strategy: &mut PixelStrategy<T>) {
        enum FlipType {
            Opening,
            Closing,
            Singular,
        }
        struct Flip {
            edge_type: FlipType,
            position: i32,
            smooth: Option<bool>,
        }

        // SAFETY: we do believe that there are at least 3 points in `vertices`.
        let ((left, top), (_right, bottom)) = vertices[..].iter().fold(
            (vertices[0].split(), vertices[0].split()),
            |((left, top), (right, bottom)), value| {
                let left = left.min(value.x());
                let right = right.max(value.x());
                let top = top.min(value.y());
                let bottom = bottom.max(value.y());
                ((left, top), (right, bottom))
            },
        );

        for y in top..=bottom {
            let segments = AngleIterator::new(vertices);

            let intersections = segments
                .map(|(a, b, c)| {
                    (scanline_segment_i32((*a, *b), y), {
                        if b.y() == y {
                            Some((b.y() > a.y()) == (b.y() < c.y()))
                        } else {
                            None
                        }
                    })
                })
                .filter(|(scan, _)| !matches!(*scan, Scan::None));

            let mut flips = Vec::new();
            for (intersection, smooth) in intersections {
                match intersection {
                    Scan::Single(a) => flips.push(Flip {
                        edge_type: FlipType::Singular,
                        position: a,
                        smooth,
                    }),
                    Scan::Inclusive(a, b) => {
                        flips.push(Flip {
                            edge_type: FlipType::Opening,
                            position: a,
                            smooth,
                        });
                        flips.push(Flip {
                            edge_type: FlipType::Closing,
                            position: b,
                            smooth,
                        });
                    }
                    Scan::None => {}
                }
            }
            flips.sort_by(|a, b| a.position.cmp(&b.position));

            let mut counter = 0;
            let mut current_left = left;
            for flip in flips {
                if counter % 2 == 1 || counter / 2 % 2 == 1 {
                    self.horizontal_line(
                        current_left + self.offset.x(),
                        flip.position + self.offset.x(),
                        y + self.offset.y(),
                        strategy,
                        0,
                    );
                }
                current_left = flip.position;

                counter += match (flip.edge_type, flip.smooth) {
                    (FlipType::Singular, Some(false)) => 2,
                    (FlipType::Opening, None) => 1,
                    (FlipType::Closing, None) => 1,
                    (FlipType::Singular, None) => 2,
                    (FlipType::Opening, Some(false)) => 1,
                    (FlipType::Closing, Some(false)) => 1,
                    _ => 0,
                };
            }
        }
    }

    fn filled_circle(&mut self, center: Vector<i32>, radius: i32, function: &mut PixelStrategy<T>) {
        self.horizontal_line(
            center.x() - radius,
            center.x() + radius,
            center.y(),
            function,
            0,
        );

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        while x < y {
            if decision > 0 {
                self.horizontal_line(center.x() - x, center.x() + x, center.y() + y, function, 0);
                self.horizontal_line(center.x() - x, center.x() + x, center.y() - y, function, 0);
                y -= 1;
                checker_y += 2;
                decision += checker_y;
            } else {
                x += 1;
                checker_x += 2;
                decision += checker_x;

                self.horizontal_line(center.x() - y, center.x() + y, center.y() + x, function, 0);
                self.horizontal_line(center.x() - y, center.x() + y, center.y() - x, function, 0);
            }
        }
    }

    fn circle(&mut self, center: Vector<i32>, radius: i32, strategy: &mut PixelStrategy<T>) {
        self.apply_strategy(center + (radius, 0), strategy);
        self.apply_strategy(center - (radius, 0), strategy);
        self.apply_strategy(center + (0, radius), strategy);
        self.apply_strategy(center - (0, radius), strategy);

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        let mut mapper = |x, y| {
            self.apply_strategy(center + (x, y), strategy);
            self.apply_strategy(center + (x, -y), strategy);
            self.apply_strategy(center + (-x, y), strategy);
            self.apply_strategy(center + (-x, -y), strategy);

            self.apply_strategy(center + (y, x), strategy);
            self.apply_strategy(center + (y, -x), strategy);
            self.apply_strategy(center + (-y, x), strategy);
            self.apply_strategy(center + (-y, -x), strategy);
        };

        while x < y - 2 {
            if decision > 0 {
                y -= 1;
                checker_y += 2;
                decision += checker_y;
            }
            x += 1;
            checker_x += 2;
            decision += checker_x;
            mapper(x, y);
        }

        if x == y - 2 {
            let x = x + 1;
            self.apply_strategy(center + (x, x), strategy);
            self.apply_strategy(center + (x, -x), strategy);
            self.apply_strategy(center + (-x, x), strategy);
            self.apply_strategy(center + (-x, -x), strategy);
        }
    }

    fn zip_map_images<O: Clone, U: Image<Pixel = O> + ?Sized>(
        &mut self,
        at: Vector<i32>,
        image: &U,
        function: &mut ImageMapper<T::Pixel, O>,
    ) where
        for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
        for<'b> <U as DesignatorRef<'b>>::PixelRef: Deref<Target = O>,
    {
        let image_start_x = if at.x() < 0 { -at.x() } else { 0 };
        let image_start_y = if at.y() < 0 { -at.y() } else { 0 };

        let image_end_x = if at.x() + image.width() >= self.target.width() {
            self.target.width() - at.x()
        } else {
            image.width()
        };
        let image_end_y = if at.y() + image.height() >= self.target.height() {
            self.target.height() - at.y()
        } else {
            image.height()
        };
        for x in image_start_x..image_end_x {
            for y in image_start_y..image_end_y {
                let step = (x, y).into();
                let pose = at + step;
                unsafe {
                    let color = Image::unsafe_pixel(image, step);
                    let pixel = function(
                        pose.split(),
                        self.target.unsafe_pixel(pose).clone(),
                        (x, y),
                        color.clone(),
                    );
                    *self.target.unsafe_pixel_mut(pose) = pixel;
                }
            }
        }
    }
}

impl<T> Paint<T, i32> for Painter<'_, T>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn pixel(&self, position: Vector<i32>) -> Option<PixelRef<'_, T>> {
        let position = self.position_i32(position);
        Image::pixel(self.target, position)
    }

    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<PixelMut<'_, T>> {
        let position = self.position_i32(position);
        ImageMut::pixel_mut(self.target, position)
    }

    fn mod_pixel<S>(&mut self, position: Vector<i32>, strategy: S)
    where
        for<'a> S: Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let position = self.position_i32(position);
        self.apply_strategy(position, &mut strategy);
    }

    fn line<'a, S>(&mut self, from: Vector<i32>, to: Vector<i32>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_i32(from);
        let to = self.position_i32(to);
        self.paint_line(from, to, &mut strategy, 0);
    }

    fn rect_f<'a, S>(&mut self, from: Vector<i32>, dimensions: Vector<i32>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_i32(from);
        self.filled_rect(from, from + dimensions - (1, 1), &mut strategy);
    }

    fn rect_b<'a, S>(&mut self, from: Vector<i32>, dimensions: Vector<i32>, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_i32(from);
        let (from, to) = (from, from + dimensions - (1, 1));
        self.horizontal_line(from.x(), to.x(), from.y(), &mut strategy, 1);
        self.horizontal_line(to.x(), from.x(), to.y(), &mut strategy, 1);
        self.vertical_line(from.x(), to.y(), from.y(), &mut strategy, 1);
        self.vertical_line(to.x(), from.y(), to.y(), &mut strategy, 1);
    }

    fn triangle_f<'a, S>(&mut self, vertices: [Vector<i32>; 3], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let vertices = vertices.map(|v| self.position_i32(v));
        self.filled_triangle(vertices, &mut strategy);
    }

    fn triangle_b<'a, S>(&mut self, vertices: [Vector<i32>; 3], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let [a, b, c] = vertices.map(|v| self.position_i32(v));
        let mut strategy = strategy.into();
        self.paint_line(a, b, &mut strategy, 1);
        self.paint_line(b, c, &mut strategy, 1);
        self.paint_line(c, a, &mut strategy, 1);
    }

    fn polygon_f<'a, S>(&mut self, vertices: &[Vector<i32>], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        self.filled_polygon_raw(vertices, &mut strategy);
    }

    fn polygon_b<'a, S>(&mut self, vertices: &[Vector<i32>], strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let skip = if vertices.len() > 2 {
            self.paint_line(
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                self.position_i32(*vertices.last().unwrap()),
                self.position_i32(vertices[0]),
                &mut strategy,
                1,
            );
            1
        } else {
            0
        };

        for window in vertices.windows(2) {
            self.paint_line(
                self.position_i32(window[0]),
                self.position_i32(window[1]),
                &mut strategy,
                skip,
            );
        }
    }

    fn circle_f<'a, S>(&mut self, center: Vector<i32>, radius: i32, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let center = self.position_i32(center);
        self.filled_circle(center, radius, &mut strategy);
    }

    fn circle_b<'a, S>(&mut self, center: Vector<i32>, radius: i32, strategy: S)
    where
        T::Pixel: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let center = self.position_i32(center);
        self.circle(center, radius, &mut strategy);
    }
}

impl<T> Painter<'_, T>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    /// Get reference to pixel.
    ///
    /// # Safety
    /// - `position + self.offset` must be in the `[0, (width, height))` range.
    pub unsafe fn pixel_unsafe(&self, position: Vector<i32>) -> PixelRef<'_, T> {
        let position = self.position_i32(position);
        unsafe { Image::unsafe_pixel(self.target, position) }
    }

    /// Get mutable reference to pixel.
    ///
    /// # Safety
    /// - `position + self.offset` must be in the `[0, (width, height))` range.
    pub unsafe fn pixel_mut_unsafe(&mut self, position: Vector<i32>) -> PixelMut<'_, T> {
        let position = self.position_i32(position);
        unsafe { ImageMut::unsafe_pixel_mut(self.target, position) }
    }

    /// Use provided function and the given image.
    pub fn image<O, U>(
        &mut self,
        at: Vector<i32>,
        image: &U,
        function: &mut ImageMapper<T::Pixel, O>,
    ) where
        U: Image<Pixel = O> + ?Sized,
        O: Clone,
        for<'a> <U as DesignatorRef<'a>>::PixelRef: Deref<Target = O>,
    {
        let at = self.position_i32(at);
        self.zip_map_images(at, image, function)
    }

    /// Use provided spatial mapper, font and mapper function to draw text.
    pub fn text<M, U, O>(
        &mut self,
        at: Vector<i32>,
        mapper: M,
        font: &dyn Getter<Index = char, Item = U>,
        text: &str,
        function: &mut ImageMapper<T::Pixel, O>,
    ) where
        M: FnMut(char, &U) -> Vector<i32>,
        U: Image<Pixel = O>,
        O: Clone,
        for<'a> <U as DesignatorRef<'a>>::PixelRef: Deref<Target = O>,
    {
        let at = self.position_i32(at);
        let mut mapper = mapper;
        for code_point in text.chars() {
            if let Some(symbol) = font.get(&code_point) {
                let local = at + mapper(code_point, symbol);
                self.zip_map_images(local, symbol, function);
            }
        }
    }
}
