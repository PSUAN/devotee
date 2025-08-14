use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Generic two-dimensional vector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vector<T> {
    x: T,
    y: T,
}

impl<T> Vector<T> {
    /// Create new vector with `x` and `y` values.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Split this vector into its components.
    pub fn split(self) -> (T, T) {
        (self.x, self.y)
    }

    /// Get the x value.
    pub fn x(self) -> T {
        self.x
    }

    /// Get the y value.
    pub fn y(self) -> T {
        self.y
    }
}

impl<T> Vector<T> {
    /// Get reference to the x value.
    pub const fn x_ref(&self) -> &T {
        &self.x
    }

    /// Get reference to the y value.
    pub const fn y_ref(&self) -> &T {
        &self.y
    }

    /// Get mutable reference to the x value.
    pub const fn x_mut(&mut self) -> &mut T {
        &mut self.x
    }

    /// Get mutable reference to the y value.
    pub const fn y_mut(&mut self) -> &mut T {
        &mut self.y
    }

    /// Apply `mapper` function to both elements, one by one, return `Vector` new with new values.
    pub fn map<F, R>(self, mapper: F) -> Vector<R>
    where
        F: FnMut(T) -> R,
    {
        let mut mapper = mapper;
        Vector {
            x: mapper(self.x),
            y: mapper(self.y),
        }
    }

    /// Convert into vector of other type by converting each element.
    pub fn into<I>(self) -> Vector<I>
    where
        T: Into<I>,
    {
        Vector {
            x: self.x.into(),
            y: self.y.into(),
        }
    }

    /// Get vector with each individual element calculated as a min of corresponding elements of `self` and `other`.
    pub fn individual_min<I>(self, other: I) -> Self
    where
        I: Into<Self>,
        T: Ord,
    {
        let other = other.into();
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self { x, y }
    }

    /// Get vector with each individual element calculated as a max of corresponding elements of `self` and `other`.
    pub fn individual_max<I>(self, other: I) -> Self
    where
        I: Into<Self>,
        T: Ord,
    {
        let other = other.into();
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self { x, y }
    }
}

impl<T> Vector<T> {
    /// Calculate the dot product between `self` and `rhs` vectors.
    pub fn dot<U, R>(self, rhs: Vector<U>) -> R
    where
        T: Mul<U, Output = R>,
        R: Add<Output = R>,
    {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Calculate two-dimensional cross product between `self` and `rhs` vectors.
    /// This, in fact, calculates the value of `z` component of the resulting vector.
    pub fn cross_2d<U, R>(self, rhs: Vector<U>) -> R
    where
        T: Mul<U, Output = R>,
        R: Sub<Output = R>,
    {
        self.x * rhs.y - self.y * rhs.x
    }
}

impl Vector<i32> {
    /// Create vector with zero values.
    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl Vector<f32> {
    /// Create vector with zero values.
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
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

impl<T> MulAssign<T> for Vector<T>
where
    T: MulAssign + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs.clone();
        self.y *= rhs;
    }
}

impl<T> Div<T> for Vector<T>
where
    T: Div<Output = T> + Clone,
{
    type Output = Self;
    fn div(self, other: T) -> Self::Output {
        Self {
            x: self.x / other.clone(),
            y: self.y / other,
        }
    }
}

impl<T> DivAssign<T> for Vector<T>
where
    T: DivAssign + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs.clone();
        self.y /= rhs;
    }
}

impl<T, U> Add<U> for Vector<T>
where
    T: Add<Output = T>,
    U: Into<Vector<T>>,
{
    type Output = Self;
    fn add(self, other: U) -> Self::Output {
        let other = other.into();
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T, U> AddAssign<U> for Vector<T>
where
    T: AddAssign,
    U: Into<Vector<T>>,
{
    fn add_assign(&mut self, rhs: U) {
        let rhs = rhs.into();
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T, U> Sub<U> for Vector<T>
where
    T: Sub<Output = T>,
    U: Into<Vector<T>>,
{
    type Output = Self;
    fn sub(self, other: U) -> Self::Output {
        let other = other.into();
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T, U> SubAssign<U> for Vector<T>
where
    T: SubAssign,
    U: Into<Vector<T>>,
{
    fn sub_assign(&mut self, rhs: U) {
        let rhs = rhs.into();
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<T, R> Neg for Vector<T>
where
    T: Neg<Output = R>,
{
    type Output = Vector<R>;
    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}
