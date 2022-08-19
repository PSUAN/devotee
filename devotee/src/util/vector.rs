use std::ops::{Add, Mul, Sub};

/// Generic two-dimensional vector.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Vector<T> {
    x: T,
    y: T,
}

impl<T> Vector<T> {
    /// Create new vector with `x` and `y` values.
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vector<T>
where
    T: Clone,
{
    /// Get x value.
    pub fn x(&self) -> T {
        self.x.clone()
    }

    /// Get y value.
    pub fn y(&self) -> T {
        self.y.clone()
    }
}

impl<T> Vector<T> {
    /// Get reference to x value.
    pub fn get_x(&self) -> &T {
        &self.x
    }

    /// Get reference to y value.
    pub fn get_y(&self) -> &T {
        &self.y
    }

    /// Get mutable reference to x value.
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.x
    }

    /// Get mutable reference to y value.
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.y
    }
}

impl<T> From<(T, T)> for Vector<T> {
    fn from(source: (T, T)) -> Self {
        Self {
            x: source.0,
            y: source.1,
        }
    }
}

impl<T> From<Vector<T>> for (T, T) {
    fn from(source: Vector<T>) -> Self {
        (source.x, source.y)
    }
}

impl<T> Mul<T> for Vector<T>
where
    T: Mul<Output = T> + Clone,
{
    type Output = Self;
    fn mul(self, other: T) -> Self::Output {
        Self {
            x: self.x * other.clone(),
            y: self.y * other,
        }
    }
}

impl<T> Add for Vector<T>
where
    T: Add<Output = T> + Clone,
{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Sub for Vector<T>
where
    T: Sub<Output = T> + Clone,
{
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
