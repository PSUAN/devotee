use crate::visual::Draw;

/// An application configuration.
pub trait Config {
    /// The root node to handle App's events.
    type Node;
    /// The converter to transform palette values into `[u8; 4]` values.
    type Converter;
    /// The input handler.
    type Input;
    /// Render target to render to.
    type RenderTarget: Draw;

    /// Provide palette converter.
    fn converter() -> Self::Converter;
    /// Provide default background color for the window.
    fn background_color() -> <Self::RenderTarget as Draw>::Pixel;
}
