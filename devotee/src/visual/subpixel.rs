use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

use crate::util::vector::Vector;
use crate::visual::util::AngleIterator;

use super::image::{DesignatorMut, DesignatorRef, PixelMut, PixelRef};
use super::{Image, ImageMut, Paint, Painter, Scan};

fn scanline_segment_f32(segment: (Vector<f32>, Vector<f32>), scanline: i32) -> Scan<i32> {
    let (from, to) = if segment.0.y() < segment.1.y() {
        (segment.0, segment.1)
    } else {
        (segment.1, segment.0)
    };

    let (delta_x, delta_y) = (to - from).split();

    if scanline < from.y().round() as i32 || scanline > to.y().round() as i32 {
        return Scan::None;
    }
    if delta_y == 0.0 {
        return Scan::Inclusive(round_to_i32(from.x()), round_to_i32(to.x())).sorted();
    }

    let steep = delta_x.abs() < delta_y.abs();
    if steep {
        let y = scanline as f32;
        let x = (from.x() * to.y() - to.x() * from.y() + delta_x * y) / delta_y;
        Scan::Single(round_to_i32(x))
    } else {
        let (left, right) = if from.x() < to.x() {
            (from.x(), to.x())
        } else {
            (to.x(), from.x())
        };

        let y = scanline as f32;
        let signum = delta_x.signum();
        let common = from.x() * to.y() - to.x() * from.y();
        let first_x = (common + delta_x * (y - 0.5 * signum)) / delta_y;
        let second_x = (common + delta_x * (y + 0.5 * signum)) / delta_y;

        let (first_x, second_x) = if first_x < second_x {
            (first_x, second_x)
        } else {
            (second_x, first_x)
        };

        Scan::Inclusive(
            round_to_i32(left.max(first_x + 0.5)),
            round_to_i32(right.min(second_x - 0.5)),
        )
        .sorted()
    }
}

fn round_to_i32(value: f32) -> i32 {
    value.round() as i32
}

impl<T, P> Painter<'_, T, f32>
where
    T: ImageMut<Pixel = P>,
    T::Pixel: Clone,
    for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn map_on_subline_offset<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        from: Vector<f32>,
        to: Vector<f32>,
        function: &mut F,
        skip: usize,
    ) {
        let offset = self.offset;
        let from = from + offset;
        let to = to + offset;

        let from_i32 = from.map(round_to_i32);
        let to_i32 = to.map(round_to_i32);

        let mut iter = from_i32.y()..=to_i32.y();
        let mut iter_rev = (to_i32.y()..=from_i32.y()).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from.y() < to.y() {
            &mut iter
        } else {
            &mut iter_rev
        };

        let rev = from.x() > to.x();

        let mut skip = skip;

        for y in iter_ref {
            let scan = scanline_segment_f32((from, to), y);
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

    fn map_on_filled_subtriangle_offset<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        vertices: [Vector<f32>; 3],
        function: &mut F,
    ) {
        let offset = self.offset;
        let mut vertex = vertices.map(|v| v + offset);
        vertex.sort_by(|a, b| a.y_ref().partial_cmp(b.y_ref()).unwrap_or(Ordering::Less));
        let [a, b, c] = vertex;
        let mut vertex_i32 = vertex.map(|v| v.map(round_to_i32));
        let [a_i32, b_i32, c_i32] = vertex_i32;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertex_i32.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.map_fast_horizontal_line_raw(
                vertex_i32[0].x(),
                vertex_i32[2].x(),
                vertex_i32[0].y(),
                function,
            );
            return;
        }

        let middle = if b_i32.y() == b_i32.y() {
            b_i32.y()
        } else {
            b_i32.y() - 1
        };

        for y in a_i32.y()..=middle {
            let left_range = scanline_segment_f32((a, b), y);
            let right_range = scanline_segment_f32((a, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.map_fast_horizontal_line_raw(left, right, y, function);
        }

        let middle = middle + 1;
        for y in middle..=c_i32.y() {
            let left_range = scanline_segment_f32((a, c), y);
            let right_range = scanline_segment_f32((b, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.map_fast_horizontal_line_raw(left, right, y, function);
        }
    }

    fn map_on_filled_sane_subpolygon_offset<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        vertices: &[Vector<f32>],
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
        let (top, bottom) = (round_to_i32(top), round_to_i32(bottom));
        let left = round_to_i32(left);
        let offset = self.offset.map(round_to_i32);

        for y in top..=bottom {
            let segments = AngleIterator::new(vertices);

            let intersections = segments
                .map(|(a, b, c)| {
                    (scanline_segment_f32((*a, *b), y), {
                        if b.y().round() as i32 == y {
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
                        current_left + offset.x(),
                        flip.position + offset.x(),
                        y + offset.y(),
                        function,
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

    fn map_on_filled_subcircle<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        center: Vector<f32>,
        radius: f32,
        function: &mut F,
    ) {
        let center = self.offset + center;
        let top = round_to_i32(center.y() - radius);
        let bottom = round_to_i32(center.y() + radius);

        let rs = radius.powi(2);
        let determine_x = |y: f32| (rs - (y - center.y()).powi(2)).sqrt();

        let mut top_x = determine_x(top as f32 - 0.5);

        for scanline in top..=bottom {
            let current_x = determine_x(scanline as f32 + 0.5);
            match (top_x, current_x) {
                (a, b) if a.is_nan() && b.is_nan() => (),
                (a, b) if a.is_nan() || b > a => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                    );
                }
                (a, b) if b.is_nan() || a >= b => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                    );
                }
                (_, _) => (),
            }
            top_x = current_x;
        }
    }

    fn map_on_subcircle<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        center: Vector<f32>,
        radius: f32,
        function: &mut F,
    ) {
        let center = self.offset + center;
        let top = round_to_i32(center.y() - radius);
        let bottom = round_to_i32(center.y() + radius);

        let rs = radius.powi(2);
        let determine_x = |y: f32| (rs - (y - center.y()).powi(2)).sqrt();

        let mut top_x = determine_x(top as f32 - 0.5);

        for scanline in top..=bottom {
            let current_x = determine_x(scanline as f32 + 0.5);
            match (top_x, current_x) {
                (a, b) if a.is_nan() && b.is_nan() => (),
                (a, b) if a.is_nan() => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                    );
                }
                (a, b) if b.is_nan() => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                    );
                }
                (a, b) if a > b => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() - b),
                        scanline,
                        function,
                    );
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() + b),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                    );
                }
                (a, b) => {
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() - a),
                        scanline,
                        function,
                    );
                    self.map_fast_horizontal_line_raw(
                        round_to_i32(center.x() + a),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                    );
                }
            }
            top_x = current_x;
        }
    }
}

impl<T, P> Paint<T, f32> for Painter<'_, T, f32>
where
    T: ImageMut<Pixel = P>,
    T::Pixel: Clone,
    for<'a> <T as DesignatorRef<'a>>::PixelRef: Deref<Target = T::Pixel>,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn pixel(&self, position: Vector<f32>) -> Option<PixelRef<'_, T>> {
        Image::pixel(self.target, (position + self.offset).map(round_to_i32))
    }

    fn pixel_mut(&mut self, position: Vector<f32>) -> Option<PixelMut<'_, T>> {
        ImageMut::pixel_mut(self.target, (position + self.offset).map(round_to_i32))
    }

    fn mod_pixel<F>(&mut self, position: Vector<f32>, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        if let Some(mut pixel) = self.pixel_mut(position) {
            *pixel = function(
                round_to_i32(position.x()),
                round_to_i32(position.y()),
                pixel.clone(),
            );
        }
    }

    fn line<F>(&mut self, from: Vector<f32>, to: Vector<f32>, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        self.map_on_subline_offset(from, to, &mut function, 0);
    }

    fn rect_f<F>(&mut self, from: Vector<f32>, dimensions: Vector<f32>, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let offset = self.offset;
        let from = from + offset;
        let to = from + dimensions;
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32));
        self.map_on_filled_rect_raw(from, to, &mut function);
    }

    fn rect_b<F>(&mut self, from: Vector<f32>, dimensions: Vector<f32>, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let offset = self.offset;
        let (from, to) = (from + offset, from + dimensions + offset - (1.0, 1.0));
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32));
        let mut function = function;
        self.map_horizontal_line_raw(from.x(), to.x(), from.y(), &mut function, 1);
        self.map_horizontal_line_raw(to.x(), from.x(), to.y(), &mut function, 1);
        self.map_vertical_line_raw(from.x(), to.y(), from.y(), &mut function, 1);
        self.map_vertical_line_raw(to.x(), from.y(), to.y(), &mut function, 1);
    }

    fn triangle_f<F>(&mut self, vertices: [Vector<f32>; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        self.map_on_filled_subtriangle_offset(vertices, &mut function);
    }

    fn triangle_b<F>(&mut self, vertices: [Vector<f32>; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let [a, b, c] = vertices;
        let mut function = function;
        self.map_on_subline_offset(a, b, &mut function, 1);
        self.map_on_subline_offset(b, c, &mut function, 1);
        self.map_on_subline_offset(c, a, &mut function, 1);
    }

    fn polygon_f<F>(&mut self, vertices: &[Vector<f32>], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        match vertices.len() {
            0 => (),
            1 => self.mod_pixel(vertices[0], function),
            2 => self.line(vertices[0], vertices[1], function),
            _ => self.map_on_filled_sane_subpolygon_offset(vertices, &mut function),
        }
    }

    fn polygon_b<F>(&mut self, vertices: &[Vector<f32>], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        if vertices.len() > 2 {
            self.map_on_subline_offset(
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                *vertices.last().unwrap(),
                vertices[0],
                &mut function,
                0,
            );
        }

        for window in vertices.windows(2) {
            self.map_on_subline_offset(window[0], window[1], &mut function, 0);
        }
    }

    fn circle_f<F>(&mut self, center: Vector<f32>, radius: f32, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        self.map_on_filled_subcircle(center, radius, &mut function);
    }

    fn circle_b<F>(&mut self, center: Vector<f32>, radius: f32, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        self.map_on_subcircle(center, radius, &mut function);
    }
}
