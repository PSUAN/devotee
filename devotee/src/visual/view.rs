use crate::util::vector::Vector;

use super::{Image, ImageMut};

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

impl<T> View<T> {
    fn deform_position(&self, position: Vector<i32>) -> Vector<i32> {
        self.flip.apply(
            self.rotation.apply(position / self.scale, &self.zone),
            &self.zone,
        ) + self.zone.origin
    }

    fn position_if_in_bounds(&self, position: Vector<i32>) -> Option<Vector<i32>> {
        let position = self.deform_position(position);
        if position.x() < self.zone.origin.x() || position.y() < self.zone.origin.y() {
            return None;
        }
        if position.x() >= self.zone.origin.x() + self.zone.dimensions.x()
            || position.y() >= self.zone.origin.y() + self.zone.dimensions.y()
        {
            return None;
        }
        Some(position)
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

    /// Consume this `View` and create another one with the scale value provided.
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
    /// Create new immutable `View` into provided `target`.
    pub fn new(target: &'image T, origin: Vector<i32>, dimensions: Vector<i32>) -> Self {
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
    /// Create new mutable `View` into provided `target`.
    pub fn new_mut(target: &'image mut T, origin: Vector<i32>, dimensions: Vector<i32>) -> Self {
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

impl<T> Image for View<&T>
where
    T: Image + ?Sized,
{
    type Pixel = T::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<T::Pixel> {
        self.target.pixel(self.position_if_in_bounds(position)?)
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> T::Pixel {
        unsafe { self.target.pixel_unchecked(self.deform_position(position)) }
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

impl<T> Image for View<&mut T>
where
    T: Image + ?Sized,
{
    type Pixel = T::Pixel;

    fn pixel(&self, position: Vector<i32>) -> Option<T::Pixel> {
        self.target.pixel(self.position_if_in_bounds(position)?)
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> T::Pixel {
        unsafe { self.target.pixel_unchecked(self.deform_position(position)) }
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

impl<T> ImageMut for View<&mut T>
where
    T: ImageMut + ?Sized,
    T::Pixel: Clone,
{
    fn set_pixel(&mut self, position: Vector<i32>, value: &T::Pixel) {
        if let Some(position) = self.position_if_in_bounds(position) {
            self.target.set_pixel(position, value);
        }
    }

    fn modify_pixel(
        &mut self,
        position: Vector<i32>,
        function: &mut dyn FnMut((i32, i32), Self::Pixel) -> Self::Pixel,
    ) {
        if let Some(position) = self.position_if_in_bounds(position) {
            self.target.modify_pixel(position, function);
        }
    }

    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &T::Pixel) {
        unsafe {
            self.target
                .set_pixel_unchecked(self.zone.origin + self.deform_position(position), value);
        }
    }

    fn clear(&mut self, color: Self::Pixel) {
        // We do believe that we are in a proper range.
        // By this time we should have already recalculated origin and dimensions to be in bounds.
        unsafe {
            for y in 0..self.zone.dimensions.y() {
                for x in 0..self.zone.dimensions.x() {
                    self.target
                        .set_pixel_unchecked(self.zone.origin + (x, y), &color);
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
