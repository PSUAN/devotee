use std::marker;

/// Generic internal-surface-aware adapter.
pub mod generic;

/// Converter from internal pixel values to the supported texel values.
pub trait Converter {
    /// Internal pixel value.
    type Pixel;

    /// The conversion result.
    type Texel;

    /// Perform the conversion.
    fn forward(&self, pixel: &Self::Pixel) -> Self::Texel;

    /// Perform the inverse conversion.
    fn inverse(&self, texel: &Self::Texel) -> Self::Pixel;
}

/// A converter that simply copies data around.
#[derive(Default)]
pub struct CopyConverter<T> {
    _marker: marker::PhantomData<T>,
}

impl<T> CopyConverter<T> {
    /// Create new copy converter instance.
    pub const fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}

impl<T> Converter for CopyConverter<T>
where
    T: Copy,
{
    type Pixel = T;
    type Texel = T;

    fn forward(&self, pixel: &Self::Pixel) -> Self::Texel {
        *pixel
    }

    fn inverse(&self, texel: &Self::Texel) -> Self::Pixel {
        *texel
    }
}
