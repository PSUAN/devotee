use std::ops::RangeInclusive;

use self::color::Color;
use crate::util::getter::Getter;
use crate::util::vector::Vector;

/// Image with dimensions unknown at compile-time.
pub mod canvas;
/// Color system used in `devotee`.
pub mod color;
/// Image with compile-time known dimensions.
pub mod sprite;

/// Collection of drawing traits and functions compiles in a single prelude.
pub mod prelude {
    pub use super::color::Color;
    pub use super::Image;
    pub use super::{draw, mix, paint, printer, stamp};
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

/// Helper draw function for pixel value combining.
/// It mixes original pixel value and provided `value`.
pub fn draw<P>(value: P) -> impl FnMut(i32, i32, P) -> P
where
    P: Clone + Color,
{
    move |_, _, pixel| Color::mix(pixel, value.clone())
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

/// Helper mixer mapper for image-to-image mapping.
/// It just mixes original pixels and pixels of stamped image.
pub fn mix<P>() -> impl FnMut(i32, i32, P, i32, i32, P) -> P
where
    P: Color,
{
    move |_, _, pixel, _, _, other| pixel.mix(other)
}

/// Helper stamper mapper for image-to-image mapping.
/// It stamps pixels of the drawn image ignoring values of the original.
pub fn stamp<P>() -> impl FnMut(i32, i32, P, i32, i32, P) -> P {
    move |_, _, _original, _, _, other| other
}

fn skip_first<F: FnMut(i32, i32, P) -> P, P>(
    function: &mut F,
) -> impl FnMut(i32, i32, P) -> P + '_ {
    let mut skip = true;
    move |x, y, pixel| {
        if skip {
            skip = false;
            pixel
        } else {
            function(x, y, pixel)
        }
    }
}

fn line_scan(from: &Vector<i32>, to: &Vector<i32>, vertical_scan: i32) -> RangeInclusive<i32> {
    let (from, to) = if from.y() > to.y() {
        (from, to)
    } else {
        (to, from)
    };

    let steep = (to.x() - from.x()).abs() <= (to.y() - from.y()).abs();
    let delta_y = to.y() - from.y();
    let delta_x = to.x() - from.x();
    if delta_y == 0 {
        return from.x().min(to.x())..=from.x().max(to.x());
    }
    if steep {
        // It is one pixel wide
        let y = vertical_scan;
        let x = ((delta_x + 1) * (y - from.y())
            + (delta_x - 1) * (y - to.y())
            + (from.x() + to.x()) * delta_y)
            / (delta_y * 2);

        x..=x
    } else {
        // It is multiple pixels wide
        let (left, right) = if from.x() < to.x() {
            (from.x(), to.x())
        } else {
            (to.x(), from.x())
        };

        let y = vertical_scan;
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

        if first_x <= second_x {
            left.max(first_x)..=right.min(second_x - 1)
        } else {
            left.max(second_x)..=right.min(first_x - 1)
        }
    }
}

/// General image trait.
pub trait Image {
    /// Pixel type of this image.
    type Pixel;
    /// Get specific pixel reference.
    fn pixel(&self, position: Vector<i32>) -> Option<&Self::Pixel>;
    /// Get specific pixel mutable reference.
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<&mut Self::Pixel>;
    /// Get specific pixel reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), [width - 1, height - 1]]
    unsafe fn pixel_unsafe(&self, position: Vector<i32>) -> &Self::Pixel;
    /// Get specific pixel mutable reference without bounds check.
    ///
    /// # Safety
    /// - position must be in range [(0, 0), [width - 1, height - 1]]
    unsafe fn pixel_mut_unsafe(&mut self, position: Vector<i32>) -> &mut Self::Pixel;
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
pub trait PaintTarget<P> {
    /// Get painter for painting.
    fn painter(&mut self) -> Painter<P>;
}

/// Painter to draw on encapsulated target.
pub struct Painter<'a, P> {
    target: &'a mut dyn Image<Pixel = P>,
    offset: Vector<i32>,
}

impl<'a, P> Painter<'a, P> {
    fn new(target: &'a mut dyn Image<Pixel = P>) -> Self {
        Self {
            target,
            offset: Vector::new(0, 0),
        }
    }

    /// Get new painter with desired offset.
    pub fn with_offset(self, offset: Vector<i32>) -> Self {
        Self { offset, ..self }
    }

    /// Set offset for this particular painter.
    pub fn set_offset(&mut self, offset: Vector<i32>) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Get offset of this painter.
    pub fn offset(&self) -> Vector<i32> {
        self.offset
    }

    /// Get mutable reference to offset in this painter.
    pub fn offset_mut(&mut self) -> &mut Vector<i32> {
        &mut self.offset
    }
}

impl<'a, P> Painter<'a, P>
where
    P: Clone,
{
    fn map_on_pixel_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        point: Vector<i32>,
        function: &mut F,
    ) {
        let point = point + self.offset;
        if let Some(pixel) = self.target.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_on_pixel_helper<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        point: Vector<i32>,
        function: &mut F,
    ) {
        if let Some(pixel) = self.target.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_on_line_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) {
        let from = from + self.offset;
        let to = to + self.offset;
        if from.x() == to.x() {
            self.map_vertical_line_helper(from.x(), from.y(), to.y(), function);
            return;
        }
        if from.y() == to.y() {
            self.map_horizontal_line_helper(from.x(), to.x(), from.y(), function);
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

        for y in iter_ref {
            let mut scan = line_scan(&from, &to, y);
            let mut scan_rev = scan.clone().rev();
            let scan: &mut dyn Iterator<Item = i32> = if rev { &mut scan_rev } else { &mut scan };

            for x in scan {
                if x >= 0 && y >= 0 && x < self.target.width() && y < self.target.height() {
                    unsafe {
                        let pose = (x, y).into();
                        let pixel = function(x, y, self.target.pixel_unsafe(pose).clone());
                        *self.target.pixel_mut_unsafe(pose) = pixel;
                    }
                }
            }
        }
    }

    fn map_vertical_line_helper<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        x: i32,
        mut from_y: i32,
        mut to_y: i32,
        function: &mut F,
    ) {
        if x < 0 || x >= self.target.width() {
            return;
        }

        from_y = from_y.clamp(0, self.target.height() - 1);
        to_y = to_y.clamp(0, self.target.height() - 1);

        let mut iter = from_y..=to_y;
        let mut iter_rev = (to_y..=from_y).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_y < to_y {
            &mut iter
        } else {
            &mut iter_rev
        };

        for y in iter_ref {
            let step = (x, y).into();
            unsafe {
                let pixel = function(x, y, self.target.pixel_unsafe(step).clone());
                *self.target.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_horizontal_line_helper<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        mut from_x: i32,
        mut to_x: i32,
        y: i32,
        function: &mut F,
    ) {
        if y < 0 || y >= self.target.height() {
            return;
        }

        from_x = from_x.clamp(0, self.target.width() - 1);
        to_x = to_x.clamp(0, self.target.width() - 1);

        let mut iter = from_x..=to_x;
        let mut iter_rev = (to_x..=from_x).rev();

        let iter_ref: &mut dyn Iterator<Item = i32> = if from_x < to_x {
            &mut iter
        } else {
            &mut iter_rev
        };

        for x in iter_ref {
            let step = (x, y).into();
            unsafe {
                let pixel = function(x, y, self.target.pixel_unsafe(step).clone());
                *self.target.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_on_filled_rect_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) {
        let from = from + self.offset;
        let to = to + self.offset;

        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = (to.x()).min(self.target.width());
        let end_y = (to.y()).min(self.target.height());

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y).into();
                unsafe {
                    let pixel = function(x, y, self.target.pixel_unsafe(step).clone());
                    *self.target.pixel_mut_unsafe(step) = pixel;
                }
            }
        }
    }

    fn map_on_filled_triangle_raw<F: FnMut(i32, i32, P) -> P>(
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
            self.map_horizontal_line_helper(vertex[0].x(), vertex[2].x(), vertex[0].y(), function);
            return;
        }

        let middle = if b.y() == c.y() { b.y() } else { b.y() - 1 };

        for y in a.y()..=middle {
            let left_range = line_scan(&a, &b, y);
            let right_range = line_scan(&a, &c, y);
            let left = *left_range.start().min(right_range.start());
            let right = *left_range.end().max(right_range.end());
            self.map_horizontal_line_helper(left, right, y, function);
        }

        let middle = middle + 1;
        for y in middle..=c.y() {
            let left_range = line_scan(&a, &c, y);
            let right_range = line_scan(&b, &c, y);
            let left = *left_range.start().min(right_range.start());
            let right = *left_range.end().max(right_range.end());
            self.map_horizontal_line_helper(left, right, y, function);
        }
    }

    fn map_on_filled_sane_polygon_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        vertices: &[Vector<i32>],
        function: &mut F,
    ) {
        // We do believe that there are at least 3 points in `vertices`.
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

        let mut segments: Vec<_> = vertices
            .windows(2)
            .map(|v| (v[0], v[1], false, false, 0..=0))
            .collect();
        segments.push((*vertices.last().unwrap(), vertices[0], false, false, 0..=0));

        for y in top..=bottom {
            segments
                .iter_mut()
                .for_each(|(a, b, intersected, was_intersected, scan)| {
                    *intersected = false;
                    *was_intersected = false;
                    *scan = line_scan(a, b, y);
                });

            let mut counter = 0;
            for x in left..=right {
                let mut should_paint = false;
                for (a, b, intersected, was_intersected, scan) in segments.iter_mut() {
                    let in_vertical_range = (y >= a.y() && y < b.y()) || (y >= b.y() && y < a.y());
                    if x >= *scan.start() && x <= *scan.end() {
                        let in_full_vertical_range =
                            (y >= a.y() && y <= b.y()) || (y >= b.y() && y <= a.y());
                        if in_full_vertical_range {
                            should_paint = true;
                        }

                        if in_vertical_range {
                            *intersected = true;
                            if !*was_intersected {
                                counter += 1;
                            }
                        }
                    } else {
                        *intersected = false;
                    }
                    *was_intersected = *intersected;

                    let in_vertex = (x == a.x() && y == a.y()) || (x == b.x() && y == b.y());
                    if in_vertex {
                        should_paint = true;
                    }
                }
                should_paint = should_paint || (counter % 2) == 1;
                if should_paint {
                    self.map_on_pixel_raw((x, y).into(), function);
                }
            }
        }
    }

    fn map_on_filled_circle_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        let center = center + self.offset;
        self.map_horizontal_line_helper(
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
                self.map_horizontal_line_helper(
                    center.x() - x,
                    center.x() + x,
                    center.y() + y,
                    function,
                );
                self.map_horizontal_line_helper(
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

                self.map_horizontal_line_helper(
                    center.x() - y,
                    center.x() + y,
                    center.y() + x,
                    function,
                );
                self.map_horizontal_line_helper(
                    center.x() - y,
                    center.x() + y,
                    center.y() - x,
                    function,
                );
            }
        }
    }

    fn map_on_circle_raw<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        let center = center + self.offset;
        self.map_on_pixel_helper(center + (radius, 0), function);
        self.map_on_pixel_helper(center - (radius, 0), function);
        self.map_on_pixel_helper(center + (0, radius), function);
        self.map_on_pixel_helper(center - (0, radius), function);

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        let mut mapper = |x, y| {
            self.map_on_pixel_helper(center + (x, y), function);
            self.map_on_pixel_helper(center + (x, -y), function);
            self.map_on_pixel_helper(center + (-x, y), function);
            self.map_on_pixel_helper(center + (-x, -y), function);

            self.map_on_pixel_helper(center + (y, x), function);
            self.map_on_pixel_helper(center + (y, -x), function);
            self.map_on_pixel_helper(center + (-y, x), function);
            self.map_on_pixel_helper(center + (-y, -x), function);
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
            self.map_on_pixel_helper(center + (x, x), function);
            self.map_on_pixel_helper(center + (x, -x), function);
            self.map_on_pixel_helper(center + (-x, x), function);
            self.map_on_pixel_helper(center + (-x, -x), function);
        }
    }

    fn zip_map_images_raw<O: Clone, F: FnMut(i32, i32, P, i32, i32, O) -> P>(
        &mut self,
        at: Vector<i32>,
        image: &dyn Image<Pixel = O>,
        function: &mut F,
    ) {
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
                    let color = Image::pixel_unsafe(image, step);
                    let pixel = function(
                        pose.x(),
                        pose.y(),
                        self.target.pixel_unsafe(pose).clone(),
                        x,
                        y,
                        color.clone(),
                    );
                    *self.target.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }

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

    /// Get reference to pixel.
    pub fn pixel<I>(&self, position: I) -> Option<&P>
    where
        I: Into<Vector<i32>>,
    {
        Image::pixel(self.target, position.into() + self.offset)
    }

    /// Get mutable reference to pixel.
    pub fn pixel_mut<I>(&mut self, position: I) -> Option<&mut P>
    where
        I: Into<Vector<i32>>,
    {
        Image::pixel_mut(self.target, position.into() + self.offset)
    }
}

impl<'a, P> Painter<'a, P>
where
    P: Clone,
{
    /// Use provided function on a pixel at given position.
    pub fn mod_pixel<I, F>(&mut self, position: I, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let position = position.into();
        if let Some(pixel) = self.pixel_mut(position) {
            *pixel = function(position.x(), position.y(), pixel.clone());
        }
    }

    /// Use provided function on each pixel in a line in inclusive range.
    pub fn line<I, F>(&mut self, from: I, to: I, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_line_raw(from, to, &mut function);
    }

    /// Use provided function on each pixel in a rectangle.
    pub fn rect_f<I, F>(&mut self, from: I, to: I, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_filled_rect_raw(from, to, &mut function);
    }

    /// Use provided function on each pixel of rectangle bounds.
    pub fn rect<I, F>(&mut self, from: I, to: I, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let (from, to) = (from.into() + self.offset, to.into() - (1, 1) + self.offset);
        let mut function = function;
        {
            let mut skip = skip_first(&mut function);
            self.map_horizontal_line_helper(from.x(), to.x(), from.y(), &mut skip);
        }
        {
            let mut skip = skip_first(&mut function);
            self.map_horizontal_line_helper(to.x(), from.x(), to.y(), &mut skip);
        }
        {
            let mut skip = skip_first(&mut function);
            self.map_vertical_line_helper(from.x(), to.y(), from.y(), &mut skip);
        }
        {
            let mut skip = skip_first(&mut function);
            self.map_vertical_line_helper(to.x(), from.y(), to.y(), &mut skip);
        }
    }

    /// Use provided function on each pixel in triangle.
    pub fn triangle_f<I, F>(&mut self, vertices: [I; 3], function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let vertex = vertices.map(Into::into);
        let mut function = function;
        self.map_on_filled_triangle_raw(vertex, &mut function);
    }

    /// Use provided function on each pixel of triangle bounds.
    pub fn triangle<I, F>(&mut self, vertices: [I; 3], function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let [a, b, c] = vertices.map(Into::into);
        let mut function = function;
        {
            let mut skip = skip_first(&mut function);
            self.line(a, b, &mut skip);
        }
        {
            let mut skip = skip_first(&mut function);
            self.line(b, c, &mut skip);
        }
        {
            let mut skip = skip_first(&mut function);
            self.line(c, a, &mut skip);
        }
    }

    /// Use provided function on each pixel in polygon.
    pub fn polygon_f<I, F>(&mut self, vertices: &[I], function: F)
    where
        I: Into<Vector<i32>> + Clone,
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        let vertices: Vec<Vector<i32>> = vertices.iter().cloned().map(Into::into).collect();
        match vertices.len() {
            0 => (),
            1 => self.mod_pixel(vertices[0], function),
            2 => self.line(vertices[0], vertices[1], function),
            _ => self.map_on_filled_sane_polygon_raw(&vertices, &mut function),
        }
    }

    /// Use provided function on each pixel of polygon bounds.
    pub fn polygon<I, F>(&mut self, vertices: &[I], function: F)
    where
        I: Into<Vector<i32>> + Clone,
        F: FnMut(i32, i32, P) -> P,
    {
        let mut function = function;
        for window in vertices.windows(2) {
            let mut skip = skip_first(&mut function);
            self.map_on_line_raw(
                window[0].clone().into(),
                window[1].clone().into(),
                &mut skip,
            );
        }
        if vertices.len() > 2 {
            let mut skip = skip_first(&mut function);
            self.map_on_line_raw(
                vertices.last().unwrap().clone().into(),
                // SAFETY: we have checked that `vertices` contain at least 3 elements.
                vertices[0].clone().into(),
                &mut skip,
            );
        }
    }

    /// Use provided function on each pixel in circle.
    pub fn circle_f<I, F>(&mut self, center: I, radius: i32, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let center = center.into();
        let mut function = function;
        self.map_on_filled_circle_raw(center, radius, &mut function);
    }

    /// Use provided function on each pixel of circle bounds.
    pub fn circle<I, F>(&mut self, center: I, radius: i32, function: F)
    where
        I: Into<Vector<i32>>,
        F: FnMut(i32, i32, P) -> P,
    {
        let center = center.into();
        let mut function = function;
        self.map_on_circle_raw(center, radius, &mut function);
    }

    /// Get reference to pixel.
    ///
    /// # Safety
    /// - `position + self.offset` must be in the `[0, (width, height))` range.
    pub unsafe fn pixel_unsafe<I>(&self, position: I) -> &P
    where
        I: Into<Vector<i32>>,
    {
        Image::pixel_unsafe(self.target, position.into() + self.offset)
    }

    /// Get mutable reference to pixel.
    ///
    /// # Safety
    /// - `position + self.offset` must be in the `[0, (width, height))` range.
    pub unsafe fn pixel_mut_unsafe<I>(&mut self, position: I) -> &mut P
    where
        I: Into<Vector<i32>>,
    {
        Image::pixel_mut_unsafe(self.target, position.into() + self.offset)
    }
}

impl<'a, P> Painter<'a, P>
where
    P: Clone,
{
    /// Use provided function and given image on this drawable.
    pub fn image<I, F, O, U>(&mut self, at: I, image: &U, function: F)
    where
        I: Into<Vector<i32>>,
        U: Image<Pixel = O>,
        O: Clone,
        F: FnMut(i32, i32, P, i32, i32, O) -> P,
    {
        let at = at.into();
        let mut function = function;
        self.zip_map_images_raw(at, image, &mut function)
    }
}

impl<'a, P> Painter<'a, P>
where
    P: Clone,
{
    /// Use provided spatial mapper, font and mapper function to draw text.
    pub fn text<I, M, U, O, F>(
        &mut self,
        at: I,
        mapper: M,
        font: &dyn Getter<Index = char, Item = U>,
        text: &str,
        function: F,
    ) where
        I: Into<Vector<i32>>,
        M: FnMut(char, &U) -> Vector<i32>,
        U: Image<Pixel = O>,
        O: Clone,
        F: FnMut(i32, i32, P, i32, i32, O) -> P,
    {
        let at = at.into();
        let mut mapper = mapper;
        let mut function = function;
        for code_point in text.chars() {
            if let Some(symbol) = font.get(&code_point) {
                let local = at + mapper(code_point, symbol);
                self.zip_map_images_raw(local, symbol, &mut function);
            }
        }
    }
}
