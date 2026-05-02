use backend::middling::Surface;
use ugly_graphics::{image, strategy};

/// An adapter that wraps [`Surface`] to provide [`Image`](`image::Image`) and
/// [`ImageMut`](`image::ImageMut`) traits.
pub struct SurfaceAdapter<'a, P> {
    surface: &'a mut dyn Surface<Texel = P>,
}

impl<'a, P> SurfaceAdapter<'a, P> {
    /// Create new [`SurfaceAdapter`] instance.
    pub fn new(surface: &'a mut dyn Surface<Texel = P>) -> Self {
        Self { surface }
    }
}

impl<P> image::Dimensions for SurfaceAdapter<'_, P> {
    fn dimensions(&self) -> (u32, u32) {
        (self.surface.width(), self.surface.height())
    }
}

impl<P> image::Image for SurfaceAdapter<'_, P> {
    type Pixel = P;

    fn pixel(&self, (x, y): (u32, u32)) -> Option<Self::Pixel> {
        self.surface.texel(x, y)
    }
}

impl<P> image::ImageMut for SurfaceAdapter<'_, P>
where
    P: Clone,
{
    type Pixel = P;

    fn set_pixel(&mut self, (x, y): (u32, u32), value: Self::Pixel) {
        self.surface.set_texel(x, y, value);
    }

    fn modify_pixel(&mut self, (x, y): (u32, u32), function: strategy::Modify<Self::Pixel>) {
        if let Some(texel) = self.surface.texel(x, y) {
            self.surface.set_texel(x, y, (function)(texel));
        }
    }

    fn set_horizontal_line(&mut self, (x, y): (u32, u32), total: u32, value: Self::Pixel) {
        for x in x..x + total {
            self.set_pixel((x, y), value.clone());
        }
    }

    fn modify_horizontal_line(
        &mut self,
        (x, y): (u32, u32),
        total: u32,
        function: strategy::Modify<Self::Pixel>,
    ) {
        for x in x..x + total {
            self.modify_pixel((x, y), function);
        }
    }

    fn set(&mut self, value: Self::Pixel) {
        self.surface.clear(value);
    }

    fn modify(&mut self, function: strategy::Modify<Self::Pixel>) {
        for y in 0..self.surface.height() {
            for x in 0..self.surface.width() {
                self.modify_pixel((x, y), function);
            }
        }
    }
}
