use std::ops::{Deref, DerefMut};

use crate::util::getter::Getter;
use crate::util::vector::Vector;
use crate::visual::util::AngleIterator;

use super::image::{DesignatorMut, DesignatorRef, PixelMut, PixelRef};
use super::{Image, ImageMut, Paint, Painter, Scan};

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

impl<T> Painter<'_, T, i32>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn map_on_line_offset<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
        skip: usize,
    ) {
        let from = from + self.offset;
        let to = to + self.offset;
        if from.x() == to.x() {
            self.map_vertical_line_raw(from.x(), from.y(), to.y(), function, skip);
            return;
        }
        if from.y() == to.y() {
            self.map_horizontal_line_raw(from.x(), to.x(), from.y(), function, skip);
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
                    self.map_on_pixel_raw(pose, function);
                } else {
                    skip -= 1;
                }
            }
        }
    }

    fn map_on_filled_triangle_offset<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        vertices: [Vector<i32>; 3],
        function: &mut F,
    ) {
        let mut vertex = vertices.map(|v| v + self.offset);
        vertex.sort_by(|a, b| a.y_ref().cmp(b.y_ref()));
        let [a, b, c] = vertex;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertex.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.map_fast_horizontal_line_raw(
                vertex[0].x(),
                vertex[2].x(),
                vertex[0].y(),
                function,
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
            self.map_fast_horizontal_line_raw(left, right, y, function);
        }

        let middle = middle + 1;
        for y in middle..=c.y() {
            let left_range = scanline_segment_i32((a, c), y);
            let right_range = scanline_segment_i32((b, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.map_fast_horizontal_line_raw(left, right, y, function);
        }
    }

    fn map_on_filled_sane_polygon_offset<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        vertices: &[Vector<i32>],
        function: &mut F,
    ) {
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
                    self.map_fast_horizontal_line_raw(
                        current_left + self.offset.x(),
                        flip.position + self.offset.x(),
                        y + self.offset.y(),
                        function,
                    );
                }
                current_left = flip.position;

                counter += match (flip.edge_type, flip.smooth) {
                    (FlipType::Singular, Some(false)) => 2,
                    (FlipType::Opening, None) => 1,
                    (FlipType::Closing, None) => 1,
                    (FlipType::Singular, None) => 2,
                    _ => 0,
                };
            }
        }
    }

    fn map_on_filled_circle_offset<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        let center = center + self.offset;
        self.map_fast_horizontal_line_raw(
            center.x() - radius,
            center.x() + radius,
            center.y(),
            function,
        );

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        while x < y {
            if decision > 0 {
                self.map_fast_horizontal_line_raw(
                    center.x() - x,
                    center.x() + x,
                    center.y() + y,
                    function,
                );
                self.map_fast_horizontal_line_raw(
                    center.x() - x,
                    center.x() + x,
                    center.y() - y,
                    function,
                );
                y -= 1;
                checker_y += 2;
                decision += checker_y;
            } else {
                x += 1;
                checker_x += 2;
                decision += checker_x;

                self.map_fast_horizontal_line_raw(
                    center.x() - y,
                    center.x() + y,
                    center.y() + x,
                    function,
                );
                self.map_fast_horizontal_line_raw(
                    center.x() - y,
                    center.x() + y,
                    center.y() - x,
                    function,
                );
            }
        }
    }

    fn map_on_circle_offset<F: FnMut(i32, i32, T::Pixel) -> T::Pixel>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        let center = center + self.offset;
        self.map_on_pixel_raw(center + (radius, 0), function);
        self.map_on_pixel_raw(center - (radius, 0), function);
        self.map_on_pixel_raw(center + (0, radius), function);
        self.map_on_pixel_raw(center - (0, radius), function);

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        let mut mapper = |x, y| {
            self.map_on_pixel_raw(center + (x, y), function);
            self.map_on_pixel_raw(center + (x, -y), function);
            self.map_on_pixel_raw(center + (-x, y), function);
            self.map_on_pixel_raw(center + (-x, -y), function);

            self.map_on_pixel_raw(center + (y, x), function);
            self.map_on_pixel_raw(center + (y, -x), function);
            self.map_on_pixel_raw(center + (-y, x), function);
            self.map_on_pixel_raw(center + (-y, -x), function);
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
            self.map_on_pixel_raw(center + (x, x), function);
            self.map_on_pixel_raw(center + (x, -x), function);
            self.map_on_pixel_raw(center + (-x, x), function);
            self.map_on_pixel_raw(center + (-x, -x), function);
        }
    }

    fn zip_map_images_offset<
        O: Clone,
        F: FnMut(i32, i32, T::Pixel, i32, i32, O) -> T::Pixel,
        U: Image<Pixel = O> + ?Sized,
    >(
        &mut self,
        at: Vector<i32>,
        image: &U,
        function: &mut F,
    ) where
        for<'b> <U as DesignatorRef<'b>>::PixelRef: Deref<Target = O>,
    {
        let at = at + self.offset;
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
                        pose.x(),
                        pose.y(),
                        self.target.unsafe_pixel(pose).clone(),
                        x,
                        y,
                        color.clone(),
                    );
                    *self.target.unsafe_pixel_mut(pose) = pixel;
                }
            }
        }
    }
}

impl<T> Paint<T, i32> for Painter<'_, T, i32>
where
    T: ImageMut,
    T::Pixel: Clone,
    for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn pixel(&self, position: Vector<i32>) -> Option<PixelRef<'_, T>> {
        Image::pixel(self.target, position + self.offset)
    }

    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<PixelMut<'_, T>> {
        ImageMut::pixel_mut(self.target, position + self.offset)
    }

    fn mod_pixel<F>(&mut self, position: Vector<i32>, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        if let Some(mut pixel) = self.pixel_mut(position) {
            *pixel = function(position.x(), position.y(), pixel.clone());
        }
    }

    fn line<F>(&mut self, from: Vector<i32>, to: Vector<i32>, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        self.map_on_line_offset(from, to, &mut function, 0);
    }

    fn rect_f<F>(&mut self, from: Vector<i32>, dimensions: Vector<i32>, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        let from = from + self.offset;
        let to = from + dimensions;
        self.map_on_filled_rect_raw(from, to, &mut function);
    }

    fn rect_b<F>(&mut self, from: Vector<i32>, dimensions: Vector<i32>, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let (from, to) = (from + self.offset, from + dimensions + self.offset - (1, 1));
        let mut function = function;
        self.map_horizontal_line_raw(from.x(), to.x(), from.y(), &mut function, 1);
        self.map_horizontal_line_raw(to.x(), from.x(), to.y(), &mut function, 1);
        self.map_vertical_line_raw(from.x(), to.y(), from.y(), &mut function, 1);
        self.map_vertical_line_raw(to.x(), from.y(), to.y(), &mut function, 1);
    }

    fn triangle_f<F>(&mut self, vertices: [Vector<i32>; 3], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        self.map_on_filled_triangle_offset(vertices, &mut function);
    }

    fn triangle_b<F>(&mut self, vertices: [Vector<i32>; 3], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let [a, b, c] = vertices;
        let mut function = function;
        self.map_on_line_offset(a, b, &mut function, 1);
        self.map_on_line_offset(b, c, &mut function, 1);
        self.map_on_line_offset(c, a, &mut function, 1);
    }

    fn polygon_f<F>(&mut self, vertices: &[Vector<i32>], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        match vertices.len() {
            0 => (),
            1 => self.mod_pixel(vertices[0], function),
            2 => self.line(vertices[0], vertices[1], function),
            _ => self.map_on_filled_sane_polygon_offset(vertices, &mut function),
        }
    }

    fn polygon_b<F>(&mut self, vertices: &[Vector<i32>], function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        let skip = if vertices.len() > 2 {
            self.map_on_line_offset(
                *vertices.last().unwrap(),
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                vertices[0],
                &mut function,
                1,
            );
            1
        } else {
            0
        };

        for window in vertices.windows(2) {
            self.map_on_line_offset(window[0], window[1], &mut function, skip);
        }
    }

    fn circle_f<F>(&mut self, center: Vector<i32>, radius: i32, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        self.map_on_filled_circle_offset(center, radius, &mut function);
    }

    fn circle_b<F>(&mut self, center: Vector<i32>, radius: i32, function: F)
    where
        F: FnMut(i32, i32, T::Pixel) -> T::Pixel,
    {
        let mut function = function;
        self.map_on_circle_offset(center, radius, &mut function);
    }
}

impl<T> Painter<'_, T, i32>
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
        Image::unsafe_pixel(self.target, position + self.offset)
    }

    /// Get mutable reference to pixel.
    ///
    /// # Safety
    /// - `position + self.offset` must be in the `[0, (width, height))` range.
    pub unsafe fn pixel_mut_unsafe(&mut self, position: Vector<i32>) -> PixelMut<'_, T> {
        ImageMut::unsafe_pixel_mut(self.target, position + self.offset)
    }

    /// Use provided function and given image on this drawable.
    pub fn image<F, O, U>(&mut self, at: Vector<i32>, image: &U, function: F)
    where
        U: Image<Pixel = O> + ?Sized,
        O: Clone,
        F: FnMut(i32, i32, T::Pixel, i32, i32, O) -> T::Pixel,
        for<'b> <U as DesignatorRef<'b>>::PixelRef: Deref<Target = O>,
    {
        let mut function = function;
        self.zip_map_images_offset(at, image, &mut function)
    }

    /// Use provided spatial mapper, font and mapper function to draw text.
    pub fn text<M, U, O, F>(
        &mut self,
        at: Vector<i32>,
        mapper: M,
        font: &dyn Getter<Index = char, Item = U>,
        text: &str,
        function: F,
    ) where
        M: FnMut(char, &U) -> Vector<i32>,
        U: Image<Pixel = O>,
        O: Clone,
        F: FnMut(i32, i32, T::Pixel, i32, i32, O) -> T::Pixel,
        for<'b> <U as DesignatorRef<'b>>::PixelRef: Deref<Target = O>,
    {
        let mut mapper = mapper;
        let mut function = function;
        for code_point in text.chars() {
            if let Some(symbol) = font.get(&code_point) {
                let local = at + mapper(code_point, symbol);
                self.zip_map_images_offset(local, symbol, &mut function);
            }
        }
    }
}
