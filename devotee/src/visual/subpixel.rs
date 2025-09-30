use std::cmp::Ordering;

use crate::util::vector::Vector;
use crate::visual::strategy::PixelStrategy;
use crate::visual::util::AngleIterator;

use super::{Image, Paint, Painter, Scan};

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

impl<T> Painter<'_, T>
where
    T: Clone,
{
    fn subline(
        &mut self,
        from: Vector<f32>,
        to: Vector<f32>,
        function: &mut PixelStrategy<T>,
        skip: usize,
    ) {
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
                    self.apply_strategy(pose, function);
                } else {
                    skip -= 1;
                }
            }
        }
    }

    fn filled_subtriangle(&mut self, vertices: [Vector<f32>; 3], strategy: &mut PixelStrategy<T>) {
        let mut vertex = vertices;
        vertex.sort_by(|a, b| a.y_ref().partial_cmp(b.y_ref()).unwrap_or(Ordering::Less));
        let [a, b, c] = vertex;
        let mut vertex_i32 = vertex.map(|v| v.map(round_to_i32));
        let [a_i32, b_i32, c_i32] = vertex_i32;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertex_i32.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.horizontal_line(
                vertex_i32[0].x()..=vertex_i32[2].x(),
                vertex_i32[0].y(),
                strategy,
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
            self.horizontal_line(left..=right, y, strategy, 0);
        }

        let middle = middle + 1;
        for y in middle..=c_i32.y() {
            let left_range = scanline_segment_f32((a, c), y);
            let right_range = scanline_segment_f32((b, c), y);
            let left = left_range
                .start_unchecked()
                .min(right_range.start_unchecked());
            let right = left_range.end_unchecked().max(right_range.end_unchecked());
            self.horizontal_line(left..=right, y, strategy, 0);
        }
    }

    fn filled_subpolygon_raw(&mut self, vertices: &[Vector<f32>], strategy: &mut PixelStrategy<T>) {
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
                    self.horizontal_line(
                        current_left + self.offset.x()..=flip.position + self.offset.x(),
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

    fn filled_subcircle(
        &mut self,
        center: Vector<f32>,
        radius: f32,
        strategy: &mut PixelStrategy<T>,
    ) {
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
                    self.horizontal_line(
                        round_to_i32(center.x() - b)..=round_to_i32(center.x() + b),
                        scanline,
                        strategy,
                        0,
                    );
                }
                (a, b) if b.is_nan() || a >= b => {
                    self.horizontal_line(
                        round_to_i32(center.x() - a)..=round_to_i32(center.x() + a),
                        scanline,
                        strategy,
                        0,
                    );
                }
                (_, _) => (),
            }
            top_x = current_x;
        }
    }

    fn subcircle(&mut self, center: Vector<f32>, radius: f32, strategy: &mut PixelStrategy<T>) {
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
                    self.horizontal_line(
                        round_to_i32(center.x() - b)..=round_to_i32(center.x() + b),
                        scanline,
                        strategy,
                        0,
                    );
                }
                (a, b) if b.is_nan() => {
                    self.horizontal_line(
                        round_to_i32(center.x() - a)..=round_to_i32(center.x() + a),
                        scanline,
                        strategy,
                        0,
                    );
                }
                (a, b) if a > b => {
                    self.horizontal_line(
                        round_to_i32(center.x() - a)..=round_to_i32(center.x() - b),
                        scanline,
                        strategy,
                        0,
                    );
                    self.horizontal_line(
                        round_to_i32(center.x() + b)..=round_to_i32(center.x() + a),
                        scanline,
                        strategy,
                        0,
                    );
                }
                (a, b) => {
                    self.horizontal_line(
                        round_to_i32(center.x() - b)..=round_to_i32(center.x() - a),
                        scanline,
                        strategy,
                        0,
                    );
                    self.horizontal_line(
                        round_to_i32(center.x() + a)..=round_to_i32(center.x() + b),
                        scanline,
                        strategy,
                        0,
                    );
                }
            }
            top_x = current_x;
        }
    }
}

impl<T> Paint<T, f32> for Painter<'_, T>
where
    T: Clone,
{
    fn pixel(&self, position: Vector<f32>) -> Option<T> {
        let position = self.position_f32(position).map(round_to_i32);
        Image::pixel(self.target, position)
    }

    fn mod_pixel<S>(&mut self, position: Vector<f32>, strategy: S)
    where
        for<'a> S: Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let position = self.position_f32(position).map(round_to_i32);
        self.apply_strategy(position, &mut strategy);
    }

    fn line<'a, S>(&mut self, from: Vector<f32>, to: Vector<f32>, strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_f32(from);
        let to = self.position_f32(to);
        self.subline(from, to, &mut strategy, 0);
    }

    fn rect_f<'a, S>(&mut self, from: Vector<f32>, dimensions: Vector<f32>, strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_f32(from);
        let to = from + dimensions;
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32) - (1, 1));
        self.filled_rect(from, to, &mut strategy);
    }

    fn rect_b<'a, S>(&mut self, from: Vector<f32>, dimensions: Vector<f32>, strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let from = self.position_f32(from);
        let (from, to) = (from, from + dimensions - (1.0, 1.0));
        let (from, to) = (from.map(round_to_i32), to.map(round_to_i32));
        self.horizontal_line(from.x()..=to.x(), from.y(), &mut strategy, 1);
        self.horizontal_line(to.x()..=from.x(), to.y(), &mut strategy, 1);
        self.vertical_line(from.x(), to.y(), from.y(), &mut strategy, 1);
        self.vertical_line(to.x(), from.y(), to.y(), &mut strategy, 1);
    }

    fn triangle_f<'a, S>(&mut self, vertices: [Vector<f32>; 3], strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let vertices = vertices.map(|v| self.position_f32(v));
        self.filled_subtriangle(vertices, &mut strategy);
    }

    fn triangle_b<'a, S>(&mut self, vertices: [Vector<f32>; 3], strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let [a, b, c] = vertices.map(|v| self.position_f32(v));
        self.subline(a, b, &mut strategy, 1);
        self.subline(b, c, &mut strategy, 1);
        self.subline(c, a, &mut strategy, 1);
    }

    fn polygon_f<'a, S>(&mut self, vertices: &[Vector<f32>], strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        match vertices.len() {
            0 => (),
            1 => self.apply_strategy(vertices[0].map(round_to_i32), &mut strategy),
            2 => self.subline(vertices[0], vertices[1], &mut strategy, 0),
            _ => self.filled_subpolygon_raw(vertices, &mut strategy),
        }
    }

    fn polygon_b<'a, S>(&mut self, vertices: &[Vector<f32>], strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let skip = if vertices.len() > 2 {
            self.subline(
                self.position_f32(vertices[0]),
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                self.position_f32(*vertices.last().unwrap()),
                &mut strategy,
                1,
            );
            1
        } else {
            0
        };

        for window in vertices.windows(2) {
            self.subline(
                self.position_f32(window[0]),
                self.position_f32(window[1]),
                &mut strategy,
                skip,
            );
        }
    }

    fn circle_f<'a, S>(&mut self, center: Vector<f32>, radius: f32, strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let center = self.position_f32(center);
        self.filled_subcircle(center, radius, &mut strategy);
    }

    fn circle_b<'a, S>(&mut self, center: Vector<f32>, radius: f32, strategy: S)
    where
        T: 'a,
        S: 'a + Into<PixelStrategy<'a, T>>,
    {
        let mut strategy = strategy.into();
        let center = self.position_f32(center);
        self.subcircle(center, radius, &mut strategy);
    }
}
