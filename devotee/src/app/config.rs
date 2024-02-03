/// An application configuration.
pub trait Config {
    /// The root to handle App's events.
    type Root;
    /// The converter to transform palette values into `u32` values.
    type Converter;
    /// The input handler.
    type Input;
    /// Render target to render to.
    type RenderTarget;
}
