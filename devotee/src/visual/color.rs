/// General color trait for color mixing.
pub trait Color {
    /// Mix two colors.
    /// The `other` is applied on top of `self`.
    fn mix(self, other: Self) -> Self;
}

/// Converter from pallette to `pixels`-compatible `[r, g, b, a]` array.
pub trait Converter {
    /// Palette to convert from.
    type Palette;
    /// Perform conversion.
    /// The output is considered to be `[r, g, b, a]` channels.
    fn convert(&self, color: &Self::Palette) -> [u8; 4];
}
