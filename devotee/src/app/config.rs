use crate::visual::color::Converter;
use crate::visual::Image;

/// An application configuration.
pub trait Config {
    /// The root to handle App's events.
    type Root;
    /// The converter to transform palette values into `u32` values.
    type Converter: Converter;
    /// The input handler.
    type Input;
    /// Render target to render to.
    type RenderTarget: Image<<Self::Converter as Converter>::Palette>;

    /// Provide palette converter.
    fn converter() -> Self::Converter;
    /// Provide default background color for the window.
    fn background_color() -> <Self::Converter as Converter>::Palette;
}
