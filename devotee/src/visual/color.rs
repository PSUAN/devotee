pub use devotee_backend::Converter;

/// General color trait for color mixing.
pub trait Color {
    /// Mix two colors.
    /// The `other` is applied on top of `self`.
    fn mix(self, other: Self) -> Self;
}
