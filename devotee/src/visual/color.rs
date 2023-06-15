/// General color trait for color mixing.
pub trait Color {
    /// Mix two colors.
    /// The `other` is applied on top of `self`.
    fn mix(self, other: Self) -> Self;
}

/// Converter from pallette value to `u32` value.
pub trait Converter {
    /// Palette to convert from.
    type Palette;
    /// Perform conversion.
    /// The output is considered to be `0x00rrggbb`.
    fn convert(&self, color: &Self::Palette) -> u32;
}
