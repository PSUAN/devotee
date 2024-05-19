use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

use crate::util::vector::Vector;

use super::{Image, Paint, Painter, Scan};

fn scanline_segment_f32(segment: (Vector<f32>, Vector<f32>), scanline: i32) -> Scan<i32> {
    let (from, to) = (segment.0, segment.1);

    let (delta_x, delta_y) = (to - from).split();

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

impl<'a, T, P> Painter<'a, T, f32>
where
    T: Image<Pixel = P>,
    <T as Image>::Pixel: Clone,
    for<'b> <T as Image>::PixelRef<'b>: Deref<Target = <T as Image>::Pixel>,
    for<'b> <T as Image>::PixelMut<'b>: DerefMut<Target = <T as Image>::Pixel>,
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
                    let pose = (x, y);
                    self.map_on_pixel_raw(pose.into(), function);
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
            self.map_horizontal_line_raw(
                vertex_i32[0].x(),
                vertex_i32[2].x(),
                vertex_i32[0].y(),
                function,
                0,
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
            self.map_horizontal_line_raw(left, right, y, function, 0);
        }

        let middle = middle + 1;
        for y in middle..=c_i32.y() {
            let left_range = scanline_segment_f32((a, c), y);
            let right_range = scanline_segment_f32((b, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.map_horizontal_line_raw(left, right, y, function, 0);
        }
    }

    fn map_on_filled_sane_subpolygon_offset<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        vertices: &[Vector<f32>],
        function: &mut F,
    ) {
        // SAFETY: we do believe that there are at least 3 points in `vertices`.
        let ((left, top), (right, bottom)) = vertices[1..].iter().fold(
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
        let (left, right) = (round_to_i32(left), round_to_i32(right));

        let mut segments = vertices
            .windows(2)
            .map(|v| (v[0], v[1]))
            .collect::<Vec<_>>();
        // SAFETY: we do believe that there are at least 3 points in `vertices`.
        segments.push((*vertices.last().unwrap(), vertices[0]));
        for y in top..=bottom {
            let mut segments = segments
                .iter()
                .filter(|(a, b)| {
                    (y >= a.y().round() as i32 && y <= b.y().round() as i32)
                        || (y >= b.y().round() as i32 && y <= a.y().round() as i32)
                })
                .map(|(a, b)| (a, b, false, false, scanline_segment_f32((*a, *b), y)))
                .collect::<Vec<_>>();

            let mut counter = false;
            for x in (left - 1)..=right {
                let mut should_paint = false;
                let mut intersections = 0;
                for (a, b, intersected, was_intersected, scan) in segments.iter_mut() {
                    if x >= scan.start_unchecked() && x <= scan.end_unchecked() {
                        should_paint = true;
                        *intersected = true;
                        if !*was_intersected
                            && (y < a.y().round() as i32 || y < b.y().round() as i32)
                        {
                            intersections += 1;
                        }
                    } else {
                        *intersected = false;
                    }
                    *was_intersected = *intersected;
                }

                if intersections % 2 == 1 {
                    counter = !counter;
                }

                if should_paint || counter {
                    self.map_on_pixel_raw(self.offset.map(round_to_i32) + (x, y), function);
                }
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
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                        0,
                    );
                }
                (a, b) if b.is_nan() || a >= b => {
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                        0,
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
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                        0,
                    );
                }
                (a, b) if b.is_nan() => {
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                        0,
                    );
                }
                (a, b) if a > b => {
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - a),
                        round_to_i32(center.x() - b),
                        scanline,
                        function,
                        0,
                    );
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() + b),
                        round_to_i32(center.x() + a),
                        scanline,
                        function,
                        0,
                    );
                }
                (a, b) => {
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() - b),
                        round_to_i32(center.x() - a),
                        scanline,
                        function,
                        0,
                    );
                    self.map_horizontal_line_raw(
                        round_to_i32(center.x() + a),
                        round_to_i32(center.x() + b),
                        scanline,
                        function,
                        0,
                    );
                }
            }
            top_x = current_x;
        }
    }
}

impl<'a, T, P, I> Paint<T, f32, P, I> for Painter<'a, T, f32>
where
    T: Image<Pixel = P>,
    <T as Image>::Pixel: Clone,
    for<'b> <T as Image>::PixelRef<'b>: Deref<Target = <T as Image>::Pixel>,
    for<'b> <T as Image>::PixelMut<'b>: DerefMut<Target = <T as Image>::Pixel>,
    I: Clone + Into<Vector<f32>>,
{
    fn pixel(&self, position: I) -> Option<<T as Image>::PixelRef<'_>> {
        Image::pixel(
            self.target,
            (position.into() + self.offset).map(round_to_i32),
        )
    }

    fn pixel_mut(&mut self, position: I) -> Option<<T as Image>::PixelMut<'_>> {
        Image::pixel_mut(
            self.target,
            (position.into() + self.offset).map(round_to_i32),
        )
    }

    fn mod_pixel<F>(&mut self, position: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let position = position.into();
        if let Some(mut pixel) = self.pixel_mut(position) {
            *pixel = function(
                round_to_i32(position.x()),
                round_to_i32(position.y()),
                pixel.clone(),
            );
        }
    }

    fn line<F>(&mut self, from: I, to: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_subline_offset(from, to, &mut function, 0);
    }

    fn rect_f<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let offset = self.offset;
        let (from, dimensions) = (from.into() + offset, dimensions.into());
        let to = from + dimensions;
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32));
        self.map_on_filled_rect_raw(from, to, &mut function);
    }

    fn rect_b<F>(&mut self, from: I, dimensions: I, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let offset = self.offset;
        let from = from.into();
        let (from, to) = (
            from + offset,
            from + dimensions.into() + offset - (1.0, 1.0),
        );
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32));
        let mut function = function;
        self.map_horizontal_line_raw(from.x(), to.x(), from.y(), &mut function, 1);
        self.map_horizontal_line_raw(to.x(), from.x(), to.y(), &mut function, 1);
        self.map_vertical_line_raw(from.x(), to.y(), from.y(), &mut function, 1);
        self.map_vertical_line_raw(to.x(), from.y(), to.y(), &mut function, 1);
    }

    fn triangle_f<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let vertex = vertices.map(Into::into);
        let mut function = function;
        self.map_on_filled_subtriangle_offset(vertex, &mut function);
    }

    fn triangle_b<F>(&mut self, vertices: [I; 3], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let [a, b, c] = vertices.map(Into::into);
        let mut function = function;
        self.map_on_subline_offset(a, b, &mut function, 1);
        self.map_on_subline_offset(b, c, &mut function, 1);
        self.map_on_subline_offset(c, a, &mut function, 1);
    }

    fn polygon_f<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let vertices = vertices.iter().cloned().map(Into::into).collect::<Vec<_>>();
        match vertices.len() {
            0 => (),
            1 => self.mod_pixel(vertices[0], function),
            2 => self.line(vertices[0], vertices[1], function),
            _ => self.map_on_filled_sane_subpolygon_offset(&vertices, &mut function),
        }
    }

    fn polygon_b<F>(&mut self, vertices: &[I], function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let skip = if vertices.len() > 2 {
            self.map_on_subline_offset(
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                vertices.last().unwrap().clone().into(),
                vertices[0].clone().into(),
                &mut function,
                1,
            );
            1
        } else {
            0
        };

        for window in vertices.windows(2) {
            self.map_on_subline_offset(
                window[0].clone().into(),
                window[1].clone().into(),
                &mut function,
                skip,
            );
        }
    }

    fn circle_f<F>(&mut self, center: I, radius: f32, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let center = center.into();
        self.map_on_filled_subcircle(center, radius, &mut function);
    }

    fn circle_b<F>(&mut self, center: I, radius: f32, function: F)
    where
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let center = center.into();
        self.map_on_subcircle(center, radius, &mut function);
    }
}
