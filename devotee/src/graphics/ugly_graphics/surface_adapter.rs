use devotee_backend::middling;
use ugly_graphics::{image, strategy};

/// An adapter that wraps [`Surface`] to provide [`Image`](`image::Image`) and
/// [`ImageMut`](`image::ImageMut`) traits.
pub struct SurfaceAdapter<'a, P> {
    surface: &'a mut dyn middling::Surface<Texel = P>,
}

impl<'a, P> SurfaceAdapter<'a, P> {
    /// Create new [`SurfaceAdapter`] instance.
    pub fn new(surface: &'a mut dyn middling::Surface<Texel = P>) -> Self {
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

impl<P> image::ImageMut<strategy::Overwrite<P>> for SurfaceAdapter<'_, P>
where
    P: Clone,
{
    fn write_pixel(
        &mut self,
        (x, y): (u32, u32),
        strategy::Overwrite(value): &strategy::Overwrite<P>,
    ) {
        self.surface.set_texel(x, y, value.clone());
    }

    fn write_horizontal_line(
        &mut self,
        (x, y): (u32, u32),
        total: u32,
        value: &strategy::Overwrite<P>,
    ) {
        for x in x..x + total {
            self.write_pixel((x, y), value);
        }
    }

    fn write(&mut self, strategy::Overwrite(value): &strategy::Overwrite<P>) {
        self.surface.clear(value.clone());
    }
}
