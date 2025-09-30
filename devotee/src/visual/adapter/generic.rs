use std::ops::{Deref, DerefMut};

use backend::middling::Surface;

use crate::util::vector::Vector;
use crate::visual::image::{Image, ImageMut};

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

impl<Surf, Convert> Image for Adapter<'_, '_, Surf, Convert>
where
    Surf: Surface + ?Sized,
    Surf::Texel: Clone,
    Convert: Converter<Texel = Surf::Texel>,
{
    type Pixel = Convert::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<Self::Pixel> {
        let (x, y) = position.split();
        if x < 0 || y < 0 {
            return None;
        }
        let (x, y) = (x as u32, y as u32);
        let texel = self.surface.texel(x, y)?;

        Some(self.converter.inverse(&texel))
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> Self::Pixel {
        let (x, y) = position.split();
        let texel = unsafe { self.surface.texel_unchecked(x as u32, y as u32) };
        self.converter.inverse(&texel)
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
    Convert: Converter<Texel = Surf::Texel>,
{
    fn set_pixel(&mut self, position: Vector<i32>, value: &Self::Pixel) {
        let color = self.converter.forward(value);
        let (x, y) = position.split();
        if let (Some(x), Some(y)) = (x.try_into().ok(), y.try_into().ok()) {
            self.surface.set_texel(x, y, color);
        }
    }

    fn modify_pixel(
        &mut self,
        position: Vector<i32>,
        function: &mut dyn FnMut((i32, i32), Self::Pixel) -> Self::Pixel,
    ) {
        let (x, y) = position.split();
        if let (Some(x), Some(y)) = (x.try_into().ok(), y.try_into().ok())
            && let Some(value) = self.surface.texel(x, y)
        {
            let value = self.converter.inverse(&value);
            let value = function(position.split(), value);
            let color = self.converter.forward(&value);
            self.surface.set_texel(x, y, color);
        }
    }

    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &Self::Pixel) {
        let (x, y) = position.map(|v| v as _).split();
        let value = self.converter.forward(value);
        unsafe {
            self.surface.set_texel_unchecked(x, y, value);
        }
    }

    fn clear(&mut self, color: Self::Pixel) {
        let color = self.converter.forward(&color);
        self.surface.clear(color);
    }
}
