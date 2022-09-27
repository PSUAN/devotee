use super::prelude::*;
use super::UnsafePixel;
use crate::util::getter::Getter;
use crate::util::vector::Vector;
use std::mem;

/// Sprite of fixed size.
#[derive(Clone, Copy, Debug)]
pub struct Sprite<P, const W: usize, const H: usize> {
    data: [[P; W]; H],
}

impl<P, const W: usize, const H: usize> Sprite<P, W, H>
where
    P: Copy,
{
    /// Create new Sprite with given color for each pixel.
    pub const fn with_color(color: P) -> Self {
        let data = [[color; W]; H];
        Self { data }
    }

    /// Create new Sprite with given data.
    pub const fn with_data(data: [[P; W]; H]) -> Self {
        Self { data }
    }
}

impl<P, const W: usize, const H: usize> Sprite<P, W, H>
where
    P: Clone,
{
    fn map_on_pixel(&mut self, point: Vector<i32>, function: &mut dyn FnMut(i32, i32, P) -> P) {
        if let Some(pixel) = self.pixel_mut(point) {
            *pixel = function(point.x(), point.y(), pixel.clone());
        }
    }

    fn map_on_line(
        &mut self,
        mut from: Vector<i32>,
        mut to: Vector<i32>,
        function: &mut dyn FnMut(i32, i32, P) -> P,
    ) {
        if from.x() == to.x() {
            self.map_vertical_line(from.x(), from.y(), to.y(), function);
            return;
        }
        if from.y() == to.y() {
            self.map_horizontal_line(from.x(), to.x(), from.y(), function);
            return;
        }

        let steep = (to.y() - from.y()).abs() > (to.x() - from.x()).abs();
        if steep {
            from = (from.y(), from.x()).into();
            to = (to.y(), to.x()).into();
        }

        if from.x() > to.x() {
            (to, from) = ((from.x(), from.y()).into(), (to.x(), to.y()).into());
        }

        let delta_x = to.x() - from.x();
        let delta_y = (to.y() - from.y()).abs();

        let mut error = delta_x / 2;
        let positive_y_step = from.y() < to.y();

        let mut current = from;

        while current.x() <= to.x() {
            let pose = if steep {
                (current.y(), current.x()).into()
            } else {
                current
            };

            error -= delta_y;
            if error < 0 {
                *current.y_mut() += if positive_y_step { 1 } else { -1 };
                error += delta_x;
            }

            *current.x_mut() += 1;

            if pose.x() >= 0 && pose.y() >= 0 && pose.x() < W as i32 && pose.y() < H as i32 {
                unsafe {
                    let pixel = function(pose.x(), pose.y(), self.pixel_unsafe(pose).clone());
                    *self.pixel_mut_unsafe(pose) = pixel;
                }
            }
        }
    }

    fn map_on_filled_rect(
        &mut self,
        from: Vector<i32>,
        to: Vector<i32>,
        function: &mut dyn FnMut(i32, i32, P) -> P,
    ) {
        let start_x = from.x().max(0);
        let start_y = from.y().max(0);
        let end_x = (to.x()).min(W as i32);
        let end_y = (to.y()).min(H as i32);

        for x in start_x..end_x {
            for y in start_y..end_y {
                let step = (x, y);
                unsafe {
                    let pixel = function(x, y, self.pixel_unsafe(step).clone());
                    *self.pixel_mut_unsafe(step) = pixel;
                }
            }
        }
    }

    fn map_on_filled_triangle<F: FnMut(i32, i32, P) -> P>(
        &mut self,
        vertex: [Vector<i32>; 3],
        function: &mut F,
    ) {
        let mut vertex = vertex;
        vertex.sort_by(|a, b| a.y().cmp(b.y_ref()));
        let [a, b, c] = vertex;

        // We are on a horizontal line.
        if a.y() == c.y() {
            vertex.sort_by(|a, b| a.x().cmp(b.x_ref()));
            self.map_horizontal_line(vertex[0].x(), vertex[2].x(), vertex[0].y(), function);
            return;
        }
        let delta_01 = b - a;
        let delta_02 = c - a;
        let delta_20 = a - c;
        let delta_21 = b - c;

        let middle = if b.y() == c.y() { b.y() } else { b.y() - 1 };

        let (mut sa, mut sb) = (0, 0);
        for y in a.y()..=middle {
            let left = a.x() + sa / delta_01.y();
            let right = a.x() + sb / delta_02.y();
            sa += delta_01.x();
            sb += delta_02.x();
            self.map_horizontal_line(left, right, y, function);
        }

        let middle = middle + 1;
        let (mut sa, mut sb) = (0, 0);
        for y in (middle..=c.y()).rev() {
            let left = c.x() + sa / delta_20.y();
            let right = c.x() + sb / delta_21.y();
            sa -= delta_20.x();
            sb -= delta_21.x();
            self.map_horizontal_line(left, right, y, function);
        }
    }

    fn map_on_filled_circle<F: FnMut(i32, i32, P) -> P>(
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
        let mut decision = 3 - 2 * radius;

        while x < y {
            x += 1;
            if decision > 0 {
                y -= 1;
                decision += 4 * (x - y) + 10;
            } else {
                decision += 4 * x + 6;
            }
            self.map_horizontal_line(center.x() - x, center.x() + x, center.y() + y, function);
            self.map_horizontal_line(center.x() - x, center.x() + x, center.y() - y, function);
            self.map_horizontal_line(center.x() - y, center.x() + y, center.y() + x, function);
            self.map_horizontal_line(center.x() - y, center.x() + y, center.y() - x, function);
        }
    }

    fn map_on_circle(
        &mut self,
        center: Vector<i32>,
        radius: i32,
        function: &mut dyn FnMut(i32, i32, P) -> P,
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

    fn pixel(&self, position: Vector<i32>) -> Option<&P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= W || y >= H {
            None
        } else {
            Some(&self.data[y][x])
        }
    }

    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<&mut P> {
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        let (x, y) = (position.x() as usize, position.y() as usize);
        if x >= W || y >= H {
            None
        } else {
            Some(&mut self.data[y][x])
        }
    }

    /// Get reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn pixel_unsafe<I: Into<Vector<i32>>>(&self, position: I) -> &P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &self.data[y][x]
    }

    /// Get mutable reference to pixel.
    /// # Safety
    /// - `position` must be in range `[0, width-1]` by `x` and `[0, height-1]` by `y`.
    unsafe fn pixel_mut_unsafe<I: Into<Vector<i32>>>(&mut self, position: I) -> &mut P {
        let position = position.into();
        let (x, y) = (position.x() as usize, position.y() as usize);
        &mut self.data[y][x]
    }

    fn map_vertical_line(
        &mut self,
        x: i32,
        mut from_y: i32,
        mut to_y: i32,
        function: &mut dyn FnMut(i32, i32, P) -> P,
    ) {
        if x < 0 || x >= W as i32 {
            return;
        }
        if from_y > to_y {
            mem::swap(&mut from_y, &mut to_y);
        }
        from_y = from_y.max(0);
        to_y = (to_y + 1).min(H as i32);
        for y in from_y..to_y {
            let step = (x, y);
            unsafe {
                let pixel = function(x, y, self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn map_horizontal_line(
        &mut self,
        mut from_x: i32,
        mut to_x: i32,
        y: i32,
        function: &mut dyn FnMut(i32, i32, P) -> P,
    ) {
        if y < 0 || y >= H as i32 {
            return;
        }
        if from_x > to_x {
            mem::swap(&mut from_x, &mut to_x);
        }
        from_x = from_x.max(0);
        to_x = (to_x + 1).min(W as i32);
        for x in from_x..to_x {
            let step = (x, y);
            unsafe {
                let pixel = function(x, y, self.pixel_unsafe(step).clone());
                *self.pixel_mut_unsafe(step) = pixel;
            }
        }
    }

    fn zip_map_images<O: Clone, F: FnMut(i32, i32, P, i32, i32, O) -> P>(
        &mut self,
        at: Vector<i32>,
        image: &dyn UnsafePixel<Vector<i32>, Pixel = O>,
        function: F,
    ) {
        let mut function = function;
        let image_start_x = if at.x() < 0 { -at.x() } else { 0 };
        let image_start_y = if at.y() < 0 { -at.y() } else { 0 };

        let image_end_x = if at.x() + image.width() as i32 >= W as i32 {
            W as i32 - at.x()
        } else {
            image.width() as i32
        };
        let image_end_y = if at.y() + image.height() as i32 >= H as i32 {
            H as i32 - at.y()
        } else {
            image.height() as i32
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

impl<P, const W: usize, const H: usize> Draw for Sprite<P, W, H>
where
    P: Copy,
{
    type Pixel = P;
    fn width(&self) -> i32 {
        W as i32
    }

    fn height(&self) -> i32 {
        H as i32
    }

    fn clear(&mut self, color: P) {
        self.data = [[color; W]; H];
    }
}

impl<P, const W: usize, const H: usize, I> Pixel<I> for Sprite<P, W, H>
where
    P: Copy,
    I: Into<Vector<i32>>,
{
    fn pixel(&self, position: I) -> Option<&P> {
        self.pixel(position.into())
    }

    fn pixel_mut(&mut self, position: I) -> Option<&mut P> {
        self.pixel_mut(position.into())
    }
}

impl<P, const W: usize, const H: usize, I, F> PixelMod<I, F> for Sprite<P, W, H>
where
    P: Copy,
    I: Into<Vector<i32>>,
    F: FnOnce(i32, i32, P) -> P,
{
    fn mod_pixel(&mut self, position: I, function: F) {
        let position = position.into();
        if let Some(pixel) = self.pixel_mut(position) {
            *pixel = function(position.x(), position.y(), *pixel);
        }
    }
}

impl<P, const W: usize, const H: usize, I, F> Line<I, F> for Sprite<P, W, H>
where
    P: Copy,
    I: Into<Vector<i32>>,
    F: FnMut(i32, i32, P) -> P,
{
    fn line(&mut self, from: I, to: I, function: F) {
        let (from, to) = (from.into(), to.into());
        let mut function = function;
        self.map_on_line(from, to, &mut function);
    }
}

impl<P, const W: usize, const H: usize, I, F> Rect<I, F> for Sprite<P, W, H>
where
    P: Copy,
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

impl<P, const W: usize, const H: usize, I, F> Triangle<I, F> for Sprite<P, W, H>
where
    P: Copy,
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

impl<P, const W: usize, const H: usize, I, F> Circle<I, F> for Sprite<P, W, H>
where
    P: Copy,
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

impl<P, const W: usize, const H: usize, I> UnsafePixel<I> for Sprite<P, W, H>
where
    P: Copy,
    I: Into<Vector<i32>>,
{
    unsafe fn pixel(&self, position: I) -> &Self::Pixel {
        self.pixel_unsafe(position)
    }

    unsafe fn pixel_mut(&mut self, position: I) -> &mut Self::Pixel {
        self.pixel_mut_unsafe(position)
    }
}

impl<P, const W: usize, const H: usize, I, O, U, F> Image<I, U, F> for Sprite<P, W, H>
where
    P: Copy,
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

impl<P, const W: usize, const H: usize, I, M, U, O, F> Tilemap<I, U, F, M> for Sprite<P, W, H>
where
    P: Copy,
    I: Into<Vector<i32>>,
    M: FnMut(usize, usize) -> Vector<i32>,
    U: UnsafePixel<Vector<i32>, Pixel = O>,
    O: Clone,
    F: FnMut(i32, i32, Self::Pixel, i32, i32, O) -> Self::Pixel,
{
    fn tilemap(
        &mut self,
        at: I,
        mapper: M,
        tiles: &dyn Getter<Index = usize, Item = U>,
        tile_data: &mut dyn Iterator<Item = usize>,
        function: F,
    ) {
        let at = at.into();
        let mut mapper = mapper;
        let mut function = function;
        for (index, tile) in tile_data.enumerate() {
            if let Some(tile_image) = tiles.get(&tile) {
                let local = at + mapper(index, tile);
                self.zip_map_images(local, tile_image, &mut function);
            }
        }
    }
}

impl<P, const W: usize, const H: usize, I, M, U, O, F> Text<I, U, F, M> for Sprite<P, W, H>
where
    P: Copy,
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
