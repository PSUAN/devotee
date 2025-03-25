use std::{
    marker,
    ops::{Deref, DerefMut},
};

use backend::middling::{Surface, TexelDesignatorMut, TexelDesignatorRef};

use crate::util::vector::Vector;

use super::image::{DesignatorMut, DesignatorRef, Image, ImageMut};

/// Surface adapter to implement basic drawing on surface.
pub struct Adapter<'a, Surf, Convert> {
    surface: &'a mut Surf,
    converter: Convert,
}

impl<'a, Surf, Convert> Adapter<'a, Surf, Convert> {
    /// Create new adapter instance.
    pub fn new(surface: &'a mut Surf, converter: Convert) -> Self {
        Self { surface, converter }
    }
}

/// Adaptor pixel reference.
pub struct AdapterRef<Convert>
where
    Convert: Converter,
{
    cache: Convert::Pixel,
}

impl<Convert> Deref for AdapterRef<Convert>
where
    Convert: Converter,
{
    type Target = Convert::Pixel;

    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

/// Mutable adaptor pixel reference.
pub struct AdapterMut<'a, Texel, TexelMut, Convert>
where
    TexelMut: DerefMut<Target = Texel>,
    Convert: Converter<Texel = Texel>,
{
    texel: TexelMut,
    converter: &'a Convert,

    pixel: Convert::Pixel,
}

impl<Texel, TexelMut, Convert> Deref for AdapterMut<'_, Texel, TexelMut, Convert>
where
    TexelMut: DerefMut<Target = Texel>,
    Convert: Converter<Texel = Texel>,
{
    type Target = Convert::Pixel;

    fn deref(&self) -> &Self::Target {
        &self.pixel
    }
}

impl<Texel, TexelMut, Convert> DerefMut for AdapterMut<'_, Texel, TexelMut, Convert>
where
    TexelMut: DerefMut<Target = Texel>,
    Convert: Converter<Texel = Texel>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pixel
    }
}

impl<Texel, TexelMut, Convert> Drop for AdapterMut<'_, Texel, TexelMut, Convert>
where
    TexelMut: DerefMut<Target = Texel>,
    Convert: Converter<Texel = Texel>,
{
    fn drop(&mut self) {
        *self.texel = self.converter.forward(&self.pixel);
    }
}

impl<Surf, Convert> DesignatorRef<'_> for Adapter<'_, Surf, Convert>
where
    Surf: Surface,
    Convert: Converter<Texel = Surf::Texel>,
{
    type PixelRef = AdapterRef<Convert>;
}

impl<'t, Surf, Convert> DesignatorMut<'t> for Adapter<'_, Surf, Convert>
where
    Surf: Surface + for<'a> TexelDesignatorRef<'a>,
    Convert: Converter<Texel = Surf::Texel>,
    for<'a> <Surf as TexelDesignatorMut<'a>>::TexelMut: DerefMut<Target = Surf::Texel>,
{
    type PixelMut =
        AdapterMut<'t, Surf::Texel, <Surf as TexelDesignatorMut<'t>>::TexelMut, Convert>;
}

impl<Surf, Convert> Image for Adapter<'_, Surf, Convert>
where
    Surf: Surface,
    Surf::Texel: Clone,
    for<'a> <Surf as TexelDesignatorRef<'a>>::TexelRef: Deref<Target = Surf::Texel>,
    Convert: Converter<Texel = Surf::Texel>,
{
    type Pixel = Convert::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<AdapterRef<Convert>> {
        let (x, y) = position.split();
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as u32, y as u32);
        let texel = self.surface.texel(x, y)?;

        let cache = self.converter.inverse(&texel);

        Some(AdapterRef { cache })
    }

    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> AdapterRef<Convert> {
        let (x, y) = position.split();
        let texel = self
            .surface
            .texel(x.try_into().unwrap(), y.try_into().unwrap())
            .unwrap();
        let cache = self.converter.inverse(&texel);

        AdapterRef { cache }
    }

    fn width(&self) -> i32 {
        self.surface.width() as _
    }

    fn height(&self) -> i32 {
        self.surface.height() as _
    }
}

impl<Surf, Convert> ImageMut for Adapter<'_, Surf, Convert>
where
    Surf: Surface,
    Surf::Texel: Clone,
    for<'a> <Surf as TexelDesignatorRef<'a>>::TexelRef: Deref<Target = Surf::Texel>,
    for<'a> <Surf as TexelDesignatorMut<'a>>::TexelMut: DerefMut<Target = Surf::Texel>,
    Convert: Converter<Texel = Surf::Texel>,
{
    fn pixel_mut(
        &mut self,
        position: Vector<i32>,
    ) -> Option<AdapterMut<'_, Surf::Texel, <Surf as TexelDesignatorMut>::TexelMut, Convert>> {
        let (x, y) = position.split();
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as u32, y as u32);
        let texel = self.surface.texel_mut(x, y)?;
        let cache = self.converter.inverse(&texel);
        let converter = &self.converter;

        Some(AdapterMut {
            texel,
            converter,
            pixel: cache,
        })
    }

    unsafe fn unsafe_pixel_mut(
        &mut self,
        position: Vector<i32>,
    ) -> AdapterMut<'_, Surf::Texel, <Surf as TexelDesignatorMut>::TexelMut, Convert> {
        let (x, y) = position.split();
        let texel = self
            .surface
            .texel_mut(x.try_into().unwrap(), y.try_into().unwrap())
            .unwrap();
        let cache = self.converter.inverse(&texel);
        let converter = &self.converter;

        AdapterMut {
            texel,
            converter,
            pixel: cache,
        }
    }

    fn clear(&mut self, color: Self::Pixel) {
        let color = self.converter.forward(&color);
        self.surface.clear(color);
    }
}

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
