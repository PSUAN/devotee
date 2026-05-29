use std::ops;

use ugly_graphics::image::{self, Dimensions};

pub use ugly_graphics as reimport;

/// An adapter between `ugly-graphics`' traits and
/// [`Surface`](`backend::middling::Surface`).
pub mod surface_adapter;

/// A helper transformer trait for surface writing purposes.
pub trait RowIterator<'a> {
    /// Specific pixel of this iterator.
    type Pixel;

    /// Get an iterator over every row.
    fn row_iterator(&'a self) -> impl Iterator<Item = impl Iterator<Item = Self::Pixel> + Clone>;
}

impl<'a, 'r, T> RowIterator<'a> for &'r T
where
    'r: 'a,
    T: RowIterator<'r>,
{
    type Pixel = T::Pixel;

    fn row_iterator(&'a self) -> impl Iterator<Item = impl Iterator<Item = T::Pixel> + Clone> {
        (*self).row_iterator()
    }
}

impl<P, const W: usize, const H: usize> RowIterator<'_> for image::sprite::Sprite<P, W, H>
where
    P: Clone,
{
    type Pixel = P;

    fn row_iterator(&self) -> impl Iterator<Item = impl Iterator<Item = P> + Clone> {
        self.data().iter().map(|line| line.iter().cloned())
    }
}

impl<'a, T, P> RowIterator<'a> for image::slice_based::SliceBased<T>
where
    T: ops::Deref<Target = [P]>,
    P: 'a + Clone,
{
    type Pixel = P;

    fn row_iterator(&'a self) -> impl Iterator<Item = impl Iterator<Item = P> + Clone> {
        let width = self.dimensions().0;

        self.data()
            .chunks(width as usize)
            .map(|line| line.iter().cloned())
    }
}

/// A helper wrapper over row iterator to map every pixel value via the mapper
/// function.
pub struct RowIteratorMapper<S, F> {
    source: S,
    mapper: F,
}

impl<F, S> RowIteratorMapper<S, F> {
    /// Create new wrapper instance.
    pub fn new(source: S, mapper: F) -> Self {
        Self { source, mapper }
    }
}

impl<'s, S, I, O, F> RowIterator<'s> for RowIteratorMapper<S, F>
where
    S: RowIterator<'s, Pixel = I>,
    F: Fn(I) -> O,
{
    type Pixel = O;

    fn row_iterator(&'s self) -> impl Iterator<Item = impl Iterator<Item = O> + Clone> {
        self.source
            .row_iterator()
            .map(|row| row.map(|pixel| (self.mapper)(pixel)))
    }
}
