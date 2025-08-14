use crate::visual::image::Image;

/// Pixel-related action strategy.
pub enum PixelStrategy<'a, T>
where
    T: Image,
{
    /// Overwrite destination pixels with the provided value.
    Overwrite(&'a T::Pixel),

    /// Apply provided function to calculate the resulting pixel value.
    Modify(&'a mut dyn FnMut((i32, i32), T::Pixel) -> T::Pixel),
}

impl<'a, T> PixelStrategy<'a, T>
where
    T: Image,
    T::Pixel: Clone,
{
    pub(super) fn apply(&mut self, position: (i32, i32), pixel: &mut T::Pixel) {
        match self {
            PixelStrategy::Overwrite(value) => *pixel = value.clone(),
            PixelStrategy::Modify(calculator) => *pixel = (calculator)(position, pixel.clone()),
        }
    }
}

impl<'a, T> From<&'a T::Pixel> for PixelStrategy<'a, T>
where
    T: Image,
{
    fn from(value: &'a T::Pixel) -> Self {
        Self::Overwrite(value)
    }
}

impl<'a, T, F> From<&'a mut F> for PixelStrategy<'a, T>
where
    T: Image,
    F: FnMut((i32, i32), T::Pixel) -> T::Pixel,
{
    fn from(value: &'a mut F) -> Self {
        Self::Modify(value)
    }
}
