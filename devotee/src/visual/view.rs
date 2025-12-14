use crate::util::vector::Vector;

use super::{Image, ImageMut};

#[derive(Clone, Copy, Debug)]
struct Zone {
    origin: Vector<i32>,
    start: Vector<i32>,
    dimensions: Vector<i32>,
    flip: Flip,
    rotation: Rotation,
    scale: i32,
}

impl Zone {
    fn limited_to(self, target_dimensions: Vector<i32>) -> Self {
        let dimensions = ((self.origin + self.dimensions).individual_min(target_dimensions)
            - self.origin)
            .individual_max(Vector::<i32>::zero());
        let start = -self.origin.individual_min(Vector::<i32>::zero());
        Self {
            dimensions,
            start,
            ..self
        }
    }

    fn deform_position(&self, position: Vector<i32>) -> Vector<i32> {
        self.flip.apply(
            self.rotation.apply(position / self.scale, self.dimensions),
            self.dimensions,
        ) + self.origin
    }

    fn position_if_in_bounds(&self, position: Vector<i32>) -> Option<Vector<i32>> {
        let position = self.deform_position(position);
        if position.x() < self.origin.x() || position.y() < self.origin.y() {
            return None;
        }
        if position.x() >= self.origin.x() + self.dimensions.x()
            || position.y() >= self.origin.y() + self.dimensions.y()
        {
            return None;
        }
        Some(position)
    }

    fn calculate_external_start(&self) -> Vector<i32> {
        match self.rotation {
            Rotation::None | Rotation::Half => self.start,
            Rotation::CW | Rotation::CCW => self.start.swapped(),
        }
    }
}

/// A view into an `Image`.
#[derive(Clone, Copy)]
pub struct View<'target, P> {
    target: &'target dyn Image<P>,
    zone: Zone,
}

impl<P> View<'_, P> {
    /// Get start hint.
    /// It represents the offset from the top-left corner to the actual data start position.
    pub fn get_start(&self) -> Vector<i32> {
        self.zone.calculate_external_start()
    }

    /// Get current flip value.
    pub fn get_flip(&self) -> Flip {
        self.zone.flip
    }

    /// Get mutable reference to the current flip value.
    pub fn flip_mut(&mut self) -> &mut Flip {
        &mut self.zone.flip
    }

    /// Consume this `View` and get another one with the flip value provided.
    pub fn with_flip(self, flip: Flip) -> Self {
        let zone = Zone { flip, ..self.zone };
        Self { zone, ..self }
    }

    /// Get current rotation value.
    pub fn get_rotation(&self) -> Rotation {
        self.zone.rotation
    }

    /// Get mutable reference to the current rotation value.
    pub fn rotation_mut(&mut self) -> &mut Rotation {
        &mut self.zone.rotation
    }

    /// Consume this `View` and get another one with the rotation value provided.
    pub fn with_rotation(self, rotation: Rotation) -> Self {
        let zone = Zone {
            rotation,
            ..self.zone
        };
        Self { zone, ..self }
    }

    /// Get current scale value.
    pub fn get_scale(&self) -> i32 {
        self.zone.scale
    }

    /// Consume this `View` and create another one with the scale value provided.
    ///
    /// # Panics
    /// Panics if `scale` is less or equal to 0.
    pub fn with_scale(self, scale: i32) -> Self {
        assert_ne!(scale, 0, "Scale can't be zero");
        assert!(scale > 0, "Scale can't be negative");
        let zone = Zone { scale, ..self.zone };
        Self { zone, ..self }
    }
}

impl<'target, P> View<'target, P> {
    /// Create new immutable `View` into provided `target`.
    pub fn new(
        target: &'target dyn Image<P>,
        origin: Vector<i32>,
        dimensions: Vector<i32>,
    ) -> Self {
        let target_dimensions = Vector::new(target.width(), target.height());
        let start = Vector::<i32>::zero();
        let flip = Flip::None;
        let rotation = Rotation::None;
        let scale = 1;
        let zone = Zone {
            origin,
            start,
            dimensions,
            flip,
            rotation,
            scale,
        }
        .limited_to(target_dimensions);

        Self { target, zone }
    }
}

/// A mutable view into an `Image`.
pub struct ViewMut<'target, P> {
    target: &'target mut dyn ImageMut<P>,
    zone: Zone,
}

impl<'image, P> ViewMut<'image, P> {
    /// Create new mutable `View` into provided `target`.
    pub fn new_mut(
        target: &'image mut dyn ImageMut<P>,
        origin: Vector<i32>,
        dimensions: Vector<i32>,
    ) -> Self {
        let target_dimensions = Vector::new(target.width(), target.height());
        let start = Vector::<i32>::zero();
        let flip = Flip::None;
        let rotation = Rotation::None;
        let scale = 1;
        let zone = Zone {
            origin,
            start,
            dimensions,
            flip,
            rotation,
            scale,
        }
        .limited_to(target_dimensions);

        Self { target, zone }
    }
}

impl<P> ViewMut<'_, P> {
    /// Get start hint.
    /// It represents the offset from the top-left corner to the actual data start position.
    pub fn get_start(&self) -> Vector<i32> {
        self.zone.calculate_external_start()
    }

    /// Get current flip value.
    pub fn get_flip(&self) -> Flip {
        self.zone.flip
    }

    /// Get mutable reference to the current flip value.
    pub fn flip_mut(&mut self) -> &mut Flip {
        &mut self.zone.flip
    }

    /// Consume this `View` and get another one with the flip value provided.
    pub fn with_flip(self, flip: Flip) -> Self {
        let zone = Zone { flip, ..self.zone };
        Self { zone, ..self }
    }

    /// Get current rotation value.
    pub fn get_rotation(&self) -> Rotation {
        self.zone.rotation
    }

    /// Get mutable reference to the current rotation value.
    pub fn rotation_mut(&mut self) -> &mut Rotation {
        &mut self.zone.rotation
    }

    /// Consume this `View` and get another one with the rotation value provided.
    pub fn with_rotation(self, rotation: Rotation) -> Self {
        let zone = Zone {
            rotation,
            ..self.zone
        };
        Self { zone, ..self }
    }

    /// Get current scale value.
    pub fn get_scale(&self) -> i32 {
        self.zone.scale
    }

    /// Consume this `View` and create another one with the scale value provided.
    ///
    /// # Panics
    /// Panics if `scale` is less or equal to 0.
    pub fn with_scale(self, scale: i32) -> Self {
        assert_ne!(scale, 0, "Scale can't be zero");
        assert!(scale > 0, "Scale can't be negative");
        let zone = Zone { scale, ..self.zone };
        Self { zone, ..self }
    }
}

impl<P> Image<P> for View<'_, P> {
    fn pixel(&self, position: Vector<i32>) -> Option<P> {
        self.target
            .pixel(self.zone.position_if_in_bounds(position)?)
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> P {
        unsafe {
            self.target
                .pixel_unchecked(self.zone.deform_position(position))
        }
    }

    fn width(&self) -> i32 {
        let dimension = match self.zone.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.x(),
            Rotation::CCW | Rotation::CW => self.zone.dimensions.y(),
        };
        dimension * self.zone.scale
    }

    fn height(&self) -> i32 {
        let dimension = match self.zone.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.y(),
            Rotation::CCW | Rotation::CW => self.zone.dimensions.x(),
        };
        dimension * self.zone.scale
    }
}

impl<P> Image<P> for ViewMut<'_, P> {
    fn pixel(&self, position: Vector<i32>) -> Option<P> {
        self.target
            .pixel(self.zone.position_if_in_bounds(position)?)
    }

    unsafe fn pixel_unchecked(&self, position: Vector<i32>) -> P {
        unsafe {
            self.target
                .pixel_unchecked(self.zone.deform_position(position))
        }
    }

    fn width(&self) -> i32 {
        let dimension = match self.zone.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.x(),
            Rotation::CCW | Rotation::CW => self.zone.dimensions.y(),
        };
        dimension * self.zone.scale
    }

    fn height(&self) -> i32 {
        let dimension = match self.zone.rotation {
            Rotation::None | Rotation::Half => self.zone.dimensions.y(),
            Rotation::CCW | Rotation::CW => self.zone.dimensions.x(),
        };
        dimension * self.zone.scale
    }
}

impl<P> ImageMut<P> for ViewMut<'_, P> {
    fn set_pixel(&mut self, position: Vector<i32>, value: &P) {
        if let Some(position) = self.zone.position_if_in_bounds(position) {
            self.target.set_pixel(position, value);
        }
    }

    fn modify_pixel(
        &mut self,
        position: Vector<i32>,
        function: &mut dyn FnMut((i32, i32), P) -> P,
    ) {
        if let Some(position) = self.zone.position_if_in_bounds(position) {
            self.target.modify_pixel(position, function);
        }
    }

    unsafe fn set_pixel_unchecked(&mut self, position: Vector<i32>, value: &P) {
        unsafe {
            self.target.set_pixel_unchecked(
                self.zone.origin + self.zone.deform_position(position),
                value,
            );
        }
    }

    fn clear(&mut self, color: P) {
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
    fn apply(&self, position: Vector<i32>, dimensions: Vector<i32>) -> Vector<i32> {
        match self {
            Flip::None => position,
            Flip::Horizontal => (dimensions.x() - position.x() - 1, position.y()).into(),
            Flip::Vertical => (position.x(), dimensions.y() - position.y() - 1).into(),
            Flip::Both => dimensions - position - (1, 1),
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
    fn apply(&self, position: Vector<i32>, dimensions: Vector<i32>) -> Vector<i32> {
        match self {
            Rotation::None => position,
            Rotation::CW => (position.y(), dimensions.y() - position.x() - 1).into(),
            Rotation::Half => dimensions - position - (1, 1),
            Rotation::CCW => (dimensions.x() - position.y() - 1, position.x()).into(),
        }
    }
}

impl<'a, I, P> From<&'a I> for View<'a, P>
where
    I: Image<P>,
{
    fn from(value: &'a I) -> Self {
        let dimensions = Vector::new(value.width(), value.height());
        View::new(value, Vector::<i32>::zero(), dimensions)
    }
}

impl<'a, P> From<&'a dyn Image<P>> for View<'a, P> {
    fn from(value: &'a dyn Image<P>) -> Self {
        let dimensions = Vector::new(value.width(), value.height());
        View::new(value, Vector::<i32>::zero(), dimensions)
    }
}

impl<'a, 'b, I, P> From<&'a mut I> for ViewMut<'b, P>
where
    'a: 'b,
    I: ImageMut<P>,
{
    fn from(value: &'a mut I) -> Self {
        let dimensions = Vector::new(value.width(), value.height());
        ViewMut::new_mut(value, Vector::<i32>::zero(), dimensions)
    }
}

impl<'a, 'b, P> From<&'a mut (dyn ImageMut<P> + 'a)> for ViewMut<'b, P>
where
    'a: 'b,
{
    fn from(value: &'a mut (dyn ImageMut<P> + 'a)) -> Self {
        let dimensions = Vector::new(value.width(), value.height());
        ViewMut::new_mut(value, Vector::<i32>::zero(), dimensions)
    }
}
