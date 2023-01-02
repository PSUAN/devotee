/// General color trait for color mixing.
pub trait Color {
    /// Mix two colors.
    /// The `other` is applied on top of `self`.
    fn mix(self, other: Self) -> Self;
}

impl Color for (f64, f64, f64, f64) {
    fn mix(self, other: Self) -> Self {
        let offset = other.0 * (1.0 - self.0);
        let alpha = self.0 + offset;
        let red = (self.1 * self.0 + other.1 * offset) / alpha;
        let green = (self.2 * self.0 + other.2 * offset) / alpha;
        let blue = (self.3 * self.0 + other.3 * offset) / alpha;
        (alpha, red, green, blue)
    }
}

impl Color for (f64, f64, f64) {
    fn mix(self, other: Self) -> Self {
        (
            (self.0 + other.0) / 2.0,
            (self.1 + other.1) / 2.0,
            (self.2 + other.2) / 2.0,
        )
    }
}

/// Converter from pallette to `pixels`-compatible `[r, g, b, a]` array.
pub trait Converter {
    /// Palette to convert from.
    type Palette;
    /// Perform conversion.
    /// The output is considered to be `[r, g, b, a]` channels.
    fn convert(&self, color: &Self::Palette) -> [u8; 4];
}
