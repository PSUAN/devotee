use std::mem;
use std::ops::RangeInclusive;

use super::{Circle, Draw, Image, Line, Pixel, PixelMod, Rect, Text, Triangle, UnsafePixel};
use crate::util::{getter::Getter, vector::Vector};

fn line_scan(from: Vector<i32>, to: Vector<i32>, vertical_scan: i32) -> RangeInclusive<i32> {
    let steep = (to.x() - from.x()).abs() < (to.y() - from.y()).abs();
    let delta_y = to.y() - from.y();
    if delta_y == 0 {
        return from.x()..=to.x();
    }
    if steep {
        // It is one pixel wide
        let y = vertical_scan;
        let x = (to.x() - from.x()) * (y - from.y()) / (delta_y) + from.x();
        x..=x
    } else {
        // It is multiple pixels wide
        let (left, right) = if from.x() < to.x() {
            (from.x(), to.x())
        } else {
            (to.x(), from.x())
        };
        let first_y = vertical_scan;
        let second_y = vertical_scan + delta_y.signum();
        let first_x = (to.x() - from.x()) * (first_y - from.y()) / delta_y + from.x();
        let second_x = (to.x() - from.x()) * (second_y - from.y()) / delta_y + from.x();
        if first_x < second_x {
            first_x..=right.min(second_x - 1)
        } else {
            left.max(second_x + 1)..=first_x
        }
    }
}

pub trait Generalization {
    type Pixel: Clone;
    fn pixel(&self, position: Vector<i32>) -> Option<&Self::Pixel>;
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<&mut Self::Pixel>;
    unsafe fn pixel_unsafe(&self, position: Vector<i32>) -> &Self::Pixel;
    unsafe fn pixel_mut_unsafe(&mut self, position: Vector<i32>) -> &mut Self::Pixel;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn clear(&mut self, color: Self::Pixel);

    fn map_on_pixel<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        point: Vector<i32>,
        function: &mut F,
    ) {
        if let Some(pixel) = self.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_on_line<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) {
        let mut from = from;
        let mut to = to;
        if from.x() == to.x() {
            self.map_vertical_line(from.x(), from.y(), to.y(), function);
            return;
        }
        if from.y() == to.y() {
            self.map_horizontal_line(from.x(), to.x(), from.y(), function);
            return;
        }

        if from.y() > to.y() {
            (from, to) = (to, from);
        }
        for y in from.y()..=to.y() {
            for x in line_scan(from, to, y) {
                if x >= 0 && y >= 0 && x < self.width() && y < self.height() {
                    unsafe {
                        let pose = (x, y).into();
                        let pixel = function(x, y, self.pixel_unsafe(pose).clone());
                        *self.pixel_mut_unsafe(pose) = pixel;
                    }
                }
            }
        }
    }

    fn map_vertical_line<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        x: i32,
        mut from_y: i32,
        mut to_y: i32,
        function: &mut F,
    ) {
        if x < 0 || x >= self.width() {
            return;
        }
        if from_y > to_y {
            mem::swap(&mut from_y, &mut to_y);
        }
        from_y = from_y.max(0);
        to_y = (to_y + 1).min(self.height());
        for y in from_y..to_y {
            let step = (x, y).into();
            unsafe {
                let pixel = function(x, y, self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_horizontal_line<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        mut from_x: i32,
        mut to_x: i32,
        y: i32,
        function: &mut F,
    ) {
        if y < 0 || y >= self.height() {
            return;
        }
        if from_x > to_x {
            mem::swap(&mut from_x, &mut to_x);
        }
        from_x = from_x.max(0);
        to_x = (to_x + 1).min(self.width());
        for x in from_x..to_x {
            let step = (x, y).into();
            unsafe {
                let pixel = function(x, y, self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_on_filled_rect<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut F,
    ) {
        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = (to.x()).min(self.width());
        let end_y = (to.y()).min(self.height());

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y).into();
                unsafe {
                    let pixel = function(x, y, self.pixel_unsafe(step).clone());
                    *self.pixel_mut_unsafe(step) = pixel;
                }
            }
        }
    }

    fn map_on_filled_triangle<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        vertices: [Vector<i32>; 3],
        function: &mut F,
    ) {
        let mut vertex = vertices;
        vertex.sort_by(|a, b| a.y_ref().cmp(b.y_ref()));
        let [a, b, c] = vertex;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertex.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.map_horizontal_line(vertex[0].x(), vertex[2].x(), vertex[0].y(), function);
            return;
        }

        let middle = if b.y() == c.y() { b.y() } else { b.y() - 1 };

        for y in a.y()..=middle {
            let left_range = line_scan(a, b, y);
            let right_range = line_scan(a, c, y);
            let left = *left_range.start().min(right_range.start());
            let right = *left_range.end().max(right_range.end());
            self.map_horizontal_line(left, right, y, function);
        }

        let middle = middle + 1;
        for y in middle..=c.y() {
            let left_range = line_scan(a, c, y);
            let right_range = line_scan(b, c, y);
            let left = *left_range.start().min(right_range.start());
            let right = *left_range.end().max(right_range.end());
            self.map_horizontal_line(left, right, y, function);
        }
    }

    fn map_on_filled_circle<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        self.map_horizontal_line(
            center.x() - radius,
            center.x() + radius,
            center.y(),
            function,
        );
        self.map_on_pixel(center + (0, radius), function);
        self.map_on_pixel(center - (0, radius), function);

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        while x < y {
            if decision > 0 {
                y -= 1;
                checker_y += 2;
                decision += checker_y;
            }
            x += 1;
            checker_x += 2;
            decision += checker_x;
            self.map_horizontal_line(center.x() - x, center.x() + x, center.y() + y, function);
            self.map_horizontal_line(center.x() - x, center.x() + x, center.y() - y, function);
            self.map_horizontal_line(center.x() - y, center.x() + y, center.y() + x, function);
            self.map_horizontal_line(center.x() - y, center.x() + y, center.y() - x, function);
        }
    }

    fn map_on_circle<F: FnMut(i32, i32, Self::Pixel) -> Self::Pixel>(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut F,
    ) {
        self.map_on_pixel(center + (radius, 0), function);
        self.map_on_pixel(center - (radius, 0), function);
        self.map_on_pixel(center + (0, radius), function);
        self.map_on_pixel(center - (0, radius), function);

        let mut x = 0;
        let mut y = radius;
        let mut decision = 1 - radius;
        let mut checker_x = 1;
        let mut checker_y = -2 * radius;

        let mut mapper = move |x, y| {
            self.map_on_pixel(center + (x, y), function);
            self.map_on_pixel(center + (x, -y), function);
            self.map_on_pixel(center + (-x, y), function);
            self.map_on_pixel(center + (-x, -y), function);

            self.map_on_pixel(center + (y, x), function);
            self.map_on_pixel(center + (y, -x), function);
            self.map_on_pixel(center + (-y, x), function);
            self.map_on_pixel(center + (-y, -x), function);
        };

        while x < y {
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
    }

    fn zip_map_images<O: Clone, F: FnMut(i32, i32, Self::Pixel, i32, i32, O) -> Self::Pixel>(
        &mut self,
        at: Vector<i32>,
        image: &dyn UnsafePixel<Vector<i32>, Pixel = O>,
        function: &mut F,
    ) {
        let image_start_x = if at.x() < 0 { -at.x() } else { 0 };
        let image_start_y = if at.y() < 0 { -at.y() } else { 0 };

        let image_end_x = if at.x() + image.width() >= self.width() {
            self.width() - at.x()
        } else {
            image.width()
        };
        let image_end_y = if at.y() + image.height() >= self.height() {
            self.height() - at.y()
        } else {
            image.height()
        };
        for x in image_start_x..image_end_x {
            for y in image_start_y..image_end_y {
                let step = (x, y).into();
                let pose = at + step;
                unsafe {
                    let color = UnsafePixel::pixel(image, step);
                    let pixel = function(
                        pose.x(),
                        pose.y(),
                        self.pixel_unsafe(pose).clone(),
                        x,
                        y,
                        color.clone(),
                    );
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }
}

impl<T, P> Draw for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
{
    type Pixel = P;

    fn width(&self) -> i32 {
        Generalization::width(self)
    }
    fn height(&self) -> i32 {
        Generalization::height(self)
    }
    fn clear(&mut self, clear_color: Self::Pixel) {
        Generalization::clear(self, clear_color)
    }
}

impl<T, P, I> Pixel<I> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
{
    fn pixel(&self, position: I) -> Option<&Self::Pixel> {
        Generalization::pixel(self, position.into())
    }

    fn pixel_mut(&mut self, position: I) -> Option<&mut Self::Pixel> {
        Generalization::pixel_mut(self, position.into())
    }
}

impl<T, P, I, F> PixelMod<I, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn mod_pixel(&mut self, position: I, function: F) {
        let mut function = function;
        let position = position.into();
        if let Some(pixel) = self.pixel_mut(position) {
            *pixel = function(position.x(), position.y(), pixel.clone());
        }
    }
}

impl<T, P, I, F> Line<I, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn line(&mut self, from: I, to: I, function: F) {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_line(from, to, &mut function);
    }
}

impl<T, P, I, F> Rect<I, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn filled_rect(&mut self, from: I, to: I, function: F) {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_filled_rect(from, to, &mut function);
    }

    fn rect(&mut self, from: I, to: I, function: F) {
        let (from, to) = (from.into(), to.into() - (1, 1));
        let mut function = function;
        self.map_horizontal_line(from.x(), to.x(), from.y(), &mut function);
        self.map_horizontal_line(from.x(), to.x(), to.y(), &mut function);
        self.map_vertical_line(from.x(), from.y(), to.y(), &mut function);
        self.map_vertical_line(to.x(), from.y(), to.y(), &mut function);
    }
}

impl<T, P, I, F> Triangle<I, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn filled_triangle(&mut self, vertex: [I; 3], function: F) {
        let vertex = vertex.map(|i| i.into());
        let mut function = function;
        self.map_on_filled_triangle(vertex, &mut function);
    }

    fn triangle(&mut self, vertex: [I; 3], function: F) {
        let [a, b, c] = vertex.map(|i| i.into());
        let mut function = function;
        self.line(a, b, &mut function);
        self.line(b, c, &mut function);
        self.line(c, a, &mut function);
    }
}

impl<T, P, I, F> Circle<I, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn filled_circle(&mut self, center: I, radius: i32, function: F) {
        let center = center.into();
        let mut function = function;
        self.map_on_filled_circle(center, radius, &mut function);
    }
    fn circle(&mut self, center: I, radius: i32, function: F) {
        let center = center.into();
        let mut function = function;
        self.map_on_circle(center, radius, &mut function);
    }
}

impl<T, P, I> UnsafePixel<I> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
{
    unsafe fn pixel(&self, position: I) -> &Self::Pixel {
        self.pixel_unsafe(position.into())
    }

    unsafe fn pixel_mut(&mut self, position: I) -> &mut Self::Pixel {
        self.pixel_mut_unsafe(position.into())
    }
}

impl<T, P, I, U, O, F> Image<I, U, F> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    U: UnsafePixel<Vector<i32>, Pixel = O>,
    O: Clone,
    F: FnMut(i32, i32, Self::Pixel, i32, i32, O) -> Self::Pixel,
{
    fn image(&mut self, at: I, image: &U, function: F) {
        let at = at.into();
        let mut function = function;
        self.zip_map_images(at, image, &mut function)
    }
}

impl<T, P, I, M, U, O, F> Text<I, U, F, M> for T
where
    T: Generalization<Pixel = P>,
    P: Clone,
    I: Into<Vector<i32>>,
    M: FnMut(char, &U) -> Vector<i32>,
    U: UnsafePixel<Vector<i32>, Pixel = O>,
    O: Clone,
    F: FnMut(i32, i32, Self::Pixel, i32, i32, O) -> Self::Pixel,
{
    fn text(
        &mut self,
        at: I,
        mapper: M,
        font: &dyn Getter<Index = char, Item = U>,
        text: &str,
        function: F,
    ) {
        let at = at.into();
        let mut mapper = mapper;
        let mut function = function;
        for code_point in text.chars() {
            if let Some(symbol) = font.get(&code_point) {
                let local = at + mapper(code_point, symbol);
                self.zip_map_images(local, symbol, &mut function);
            }
        }
    }
}
