use std::ops::{Deref, DerefMut};

use backend::middling::{Surface, TexelDesignatorMut, TexelDesignatorRef};

use crate::util::vector::Vector;
use crate::visual::image::{DesignatorMut, DesignatorRef, Image, ImageMut};

use super::Converter;

/// Surface adapter to implement basic drawing on surface.
pub struct Adapter<'a, 'b, Surf, Convert>
where
    Surf: ?Sized,
{
    surface: &'a mut Surf,
    converter: &'b Convert,
}

impl<'a, 'b, Surf, Convert> Adapter<'a, 'b, Surf, Convert>
where
    Surf: ?Sized,
{
    /// Create new adapter instance.
    pub fn new(surface: &'a mut Surf, converter: &'b Convert) -> Self {
        Self { surface, converter }
    }
}

/// Adapter pixel reference.
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

/// Mutable adapter pixel reference.
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

impl<Surf, Convert> DesignatorRef<'_> for Adapter<'_, '_, Surf, Convert>
where
    Surf: Surface + ?Sized,
    Convert: Converter<Texel = Surf::Texel>,
{
    type PixelRef = AdapterRef<Convert>;
}

impl<'t, Surf, Convert> DesignatorMut<'t> for Adapter<'_, '_, Surf, Convert>
where
    Surf: Surface + for<'a> TexelDesignatorRef<'a> + ?Sized,
    Convert: Converter<Texel = Surf::Texel>,
    for<'a> <Surf as TexelDesignatorMut<'a>>::TexelMut: DerefMut<Target = Surf::Texel>,
{
    type PixelMut =
        AdapterMut<'t, Surf::Texel, <Surf as TexelDesignatorMut<'t>>::TexelMut, Convert>;
}

impl<Surf, Convert> Image for Adapter<'_, '_, Surf, Convert>
where
    Surf: Surface + ?Sized,
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
        let texel = unsafe { self.surface.unsafe_texel(x as u32, y as u32) };
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

impl<Surf, Convert> ImageMut for Adapter<'_, '_, Surf, Convert>
where
    Surf: Surface + ?Sized,
    Surf::Texel: Clone,
    for<'a> <Surf as TexelDesignatorRef<'a>>::TexelRef: Deref<Target = Surf::Texel>,
    for<'a> <Surf as TexelDesignatorMut<'a>>::TexelMut: DerefMut<Target = Surf::Texel>,
    Convert: Converter<Texel = Surf::Texel>,
{
    fn pixel_mut(
        &mut self,
        position: Vector<i32>,
    ) -> Option<AdapterMut<'_, Surf::Texel, <Surf as TexelDesignatorMut<'_>>::TexelMut, Convert>>
    {
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
    ) -> AdapterMut<'_, Surf::Texel, <Surf as TexelDesignatorMut<'_>>::TexelMut, Convert> {
        let (x, y) = position.split();
        let texel = unsafe { self.surface.unsafe_texel_mut(x as u32, y as u32) };
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
