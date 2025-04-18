use std::ops::DerefMut;

use crate::util::vector::Vector;

use super::image::{DesignatorMut, DesignatorRef, Dimensions, PixelMut, PixelRef};
use super::{FastHorizontalWriter, Image, ImageMut};

/// Trait that provides an immutable two-dimensional view.
pub trait ViewProvider {
    /// Get an immutable view into `Self`.
    /// Resulting `View`'s origin and dimensions are cropped to the image automatically.
    fn view(&self, origin: Vector<i32>, dimensions: Vector<i32>) -> View<&Self>;
}

impl<I> ViewProvider for I
where
    I: Image,
{
    fn view(&self, origin: Vector<i32>, dimensions: Vector<i32>) -> View<&Self> {
        View::<&Self>::new(self, origin, dimensions)
    }
}

/// Trait that provides a mutable two-dimensional view.
pub trait ViewMutProvider {
    /// Get a mutable view into this `Image`.
    /// Resulting `View`'s origin and dimensions are cropped to the image automatically.
    fn view_mut(&mut self, origin: Vector<i32>, dimensions: Vector<i32>) -> View<&mut Self>;
}

impl<I> ViewMutProvider for I
where
    I: ImageMut,
{
    fn view_mut(&mut self, origin: Vector<i32>, dimensions: Vector<i32>) -> View<&mut Self> {
        View::<&mut Self>::new(self, origin, dimensions)
    }
}

#[derive(Clone, Copy, Debug)]
struct Zone {
    origin: Vector<i32>,
    dimensions: Vector<i32>,
}

/// A view into an `Image`.
#[derive(Clone, Copy, Debug)]
pub struct View<T> {
    target: T,
    zone: Zone,
    flip: Flip,
    rotation: Rotation,
    scale: i32,
}

fn calculate_origin_and_dimensions(
    original_dimensions: Vector<i32>,
    origin: Vector<i32>,
    dimensions: Vector<i32>,
) -> (Vector<i32>, Vector<i32>) {
    let origin_in_bounds = origin.individual_max((0, 0));
    let end_in_bounds = (origin + dimensions).individual_min(original_dimensions);
    let dimensions = (end_in_bounds - origin_in_bounds).individual_max((0, 0));
    (origin_in_bounds, dimensions)
}

impl<T> View<T> {
    fn deform_position(&self, position: Vector<i32>) -> Vector<i32> {
        self.flip.apply(
            self.rotation.apply(position / self.scale, &self.zone),
            &self.zone,
        )
    }

    fn position_if_in_bounds(&self, position: Vector<i32>) -> Option<Vector<i32>> {
        let position = self.deform_position(position);
        if position.x() < 0 || position.y() < 0 {
            return None;
        }
        if position.x() >= self.zone.dimensions.x() || position.y() >= self.zone.dimensions.y() {
            return None;
        }
        Some(self.zone.origin + position)
    }

    /// Get current flip value.
    pub fn get_flip(&self) -> Flip {
        self.flip
    }

    /// Get mutable reference to the current flip value.
    pub fn flip_mut(&mut self) -> &mut Flip {
        &mut self.flip
    }

    /// Consume this `View` and get another one with the flip value provided.
    pub fn with_flip(self, flip: Flip) -> Self {
        Self { flip, ..self }
    }

    /// Get current rotation value.
    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }

    /// Get mutable reference to the current rotation value.
    pub fn rotation_mut(&mut self) -> &mut Rotation {
        &mut self.rotation
    }

    /// Consume this `View` and get another one with the rotation value provided.
    pub fn with_rotation(self, rotation: Rotation) -> Self {
        Self { rotation, ..self }
    }

    /// Get current scale value.
    pub fn get_scale(&self) -> i32 {
        self.scale
    }

    /// Consume this `View` and get another one with the scale value provided.
    ///
    /// # Panics
    /// Panics if `scale` is less or equal to 0.
    pub fn with_scale(self, scale: i32) -> Self {
        assert_ne!(scale, 0, "Scale can't be zero");
        assert!(scale > 0, "Scale can't be negative");
        Self { scale, ..self }
    }
}

impl<'image, T> View<&'image T>
where
    T: Image + ?Sized,
{
    pub(super) fn new(target: &'image T, origin: Vector<i32>, dimensions: Vector<i32>) -> Self {
        let (origin, dimensions) =
            calculate_origin_and_dimensions(target.dimensions(), origin, dimensions);
        let zone = Zone { origin, dimensions };
        let flip = Flip::None;
        let rotation = Rotation::None;
        let scale = 1;

        Self {
            target,
            zone,
            flip,
            rotation,
            scale,
        }
    }
}

impl<'image, T> View<&'image mut T>
where
    T: Image + ?Sized,
{
    pub(super) fn new(target: &'image mut T, origin: Vector<i32>, dimensions: Vector<i32>) -> Self {
        let (origin, dimensions) =
            calculate_origin_and_dimensions(target.dimensions(), origin, dimensions);
        let zone = Zone { origin, dimensions };
        let flip = Flip::None;
        let rotation = Rotation::None;
        let scale = 1;

        Self {
            target,
            zone,
            flip,
            rotation,
            scale,
        }
    }
}

impl<'a, T> DesignatorRef<'a> for View<&T>
where
    T: DesignatorRef<'a> + ?Sized,
{
    type PixelRef = T::PixelRef;
}

impl<T> Image for View<&T>
where
    T: Image + ?Sized,
{
    type Pixel = T::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<PixelRef<'_, Self>> {
        self.target.pixel(self.position_if_in_bounds(position)?)
    }

    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> PixelRef<'_, Self> {
        unsafe {
            self.target
                .unsafe_pixel(self.zone.origin + self.deform_position(position))
        }
    }

    fn width(&self) -> i32 {
        match self.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.x() * self.scale,
            Rotation::CCW | Rotation::CW => self.zone.dimensions.y() * self.scale,
        }
    }

    fn height(&self) -> i32 {
        match self.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.y() * self.scale,
            Rotation::CCW | Rotation::CW => self.zone.dimensions.x() * self.scale,
        }
    }
}

impl<'a, T> DesignatorRef<'a> for View<&mut T>
where
    T: DesignatorRef<'a> + ?Sized,
{
    type PixelRef = T::PixelRef;
}

impl<T> Image for View<&mut T>
where
    T: Image + ?Sized,
{
    type Pixel = T::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<PixelRef<'_, Self>> {
        self.target.pixel(self.position_if_in_bounds(position)?)
    }

    unsafe fn unsafe_pixel(&self, position: Vector<i32>) -> PixelRef<'_, Self> {
        unsafe {
            self.target
                .unsafe_pixel(self.zone.origin + self.deform_position(position))
        }
    }

    fn width(&self) -> i32 {
        match self.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.x() * self.scale,
            Rotation::CCW | Rotation::CW => self.zone.dimensions.y() * self.scale,
        }
    }

    fn height(&self) -> i32 {
        match self.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.y() * self.scale,
            Rotation::CCW | Rotation::CW => self.zone.dimensions.x() * self.scale,
        }
    }
}

impl<'a, T> DesignatorMut<'a> for View<&mut T>
where
    T: DesignatorMut<'a> + ?Sized,
{
    type PixelMut = T::PixelMut;
}

impl<T> ImageMut for View<&mut T>
where
    T: ImageMut + ?Sized,
    T::Pixel: Clone,
    for<'a> <T as DesignatorMut<'a>>::PixelMut: DerefMut<Target = T::Pixel>,
{
    fn pixel_mut(&mut self, position: Vector<i32>) -> Option<PixelMut<'_, Self>> {
        self.target.pixel_mut(self.position_if_in_bounds(position)?)
    }

    unsafe fn unsafe_pixel_mut(&mut self, position: Vector<i32>) -> PixelMut<'_, Self> {
        unsafe {
            self.target
                .unsafe_pixel_mut(self.zone.origin + self.deform_position(position))
        }
    }

    fn clear(&mut self, color: Self::Pixel) {
        if self
            .target
            .fast_horizontal_writer()
            .map(|mut writer| {
                for y in 0..self.zone.dimensions.y() {
                    writer.write_line(
                        self.zone.origin.x()
                            ..=(self.zone.origin.x() + self.zone.dimensions.x() - 1),
                        self.zone.origin.y() + y,
                        &mut |_, _, _| color.clone(),
                    );
                }
            })
            .is_none()
        {
            // We do believe that we are in a proper range.
            // By this time we should have already recalculated origin and dimensions to be in bounds.
            unsafe {
                for y in 0..self.zone.dimensions.y() {
                    for x in 0..self.zone.dimensions.x() {
                        *self.target.unsafe_pixel_mut(self.zone.origin + (x, y)) = color.clone();
                    }
                }
            }
        }
    }
}

/// Flip transform applied to a view.
#[derive(Clone, Copy, Debug)]
pub enum Flip {
    /// No flip occurs.
    None,
    /// There is a horizontal flip.
    Horizontal,
    /// There is a vertical flip.
    Vertical,
    /// Both flips occur, effectively rotating by 180 degrees.
    Both,
}

impl From<u8> for Flip {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Flip::None,
            1 => Flip::Horizontal,
            2 => Flip::Vertical,
            3 => Flip::Both,
            _ => unreachable!(),
        }
    }
}

impl Flip {
    fn apply(&self, position: Vector<i32>, zone: &Zone) -> Vector<i32> {
        match self {
            Flip::None => position,
            Flip::Horizontal => (zone.dimensions.x() - position.x() - 1, position.y()).into(),
            Flip::Vertical => (position.x(), zone.dimensions.y() - position.y() - 1).into(),
            Flip::Both => zone.dimensions - position - (1, 1),
        }
    }
}

/// Rotation transform applied to a view.
#[derive(Clone, Copy, Debug)]
pub enum Rotation {
    /// No rotation occurs.
    None,
    /// Counterclockwise rotation by 90 degrees occurs.
    CCW,
    /// Rotation by 180 degrees occurs.
    Half,
    /// Clockwise rotation by 90 degrees occurs.
    CW,
}

impl From<u8> for Rotation {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => Rotation::None,
            1 => Rotation::CCW,
            2 => Rotation::Half,
            3 => Rotation::CW,
            _ => unreachable!(),
        }
    }
}

impl Rotation {
    fn apply(&self, position: Vector<i32>, zone: &Zone) -> Vector<i32> {
        match self {
            Rotation::None => position,
            Rotation::CW => (position.y(), zone.dimensions.y() - position.x() - 1).into(),
            Rotation::Half => zone.dimensions - position - (1, 1),
            Rotation::CCW => (zone.dimensions.x() - position.y() - 1, position.x()).into(),
        }
    }
}
