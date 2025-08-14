use std::collections::HashMap;
use std::hash::Hash;

/// Getting value reference from collection.
pub trait Getter {
    /// Indexing type for collection.
    type Index;
    /// The type of element being stored.
    type Item;
    /// Try retrieving value reference from collection by index.
    fn get(&self, index: &Self::Index) -> Option<&Self::Item>;
}

/// Getting value mutable reference from collection.
pub trait GetterMut: Getter {
    /// Try retrieving mutable value reference from collection by index.
    fn get_mut(&mut self, index: &Self::Index) -> Option<&mut Self::Item>;
}

impl<T> Getter for &[T] {
    type Index = usize;
    type Item = T;
    fn get(&self, index: &Self::Index) -> Option<&Self::Item> {
        <[T]>::get(self, *index)
    }
}

impl<T> Getter for &mut [T] {
    type Index = usize;
    type Item = T;
    fn get(&self, index: &Self::Index) -> Option<&Self::Item> {
        <[T]>::get(self, *index)
    }
}

impl<T> GetterMut for &mut [T] {
    fn get_mut(&mut self, index: &Self::Index) -> Option<&mut Self::Item> {
        <[T]>::get_mut(self, *index)
    }
}

impl<K, V> Getter for HashMap<K, V>
where
    K: Hash + Eq,
{
    type Index = K;
    type Item = V;
    fn get(&self, index: &Self::Index) -> Option<&Self::Item> {
        HashMap::get(self, index)
    }
}

impl<K, V> GetterMut for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn get_mut(&mut self, index: &Self::Index) -> Option<&mut Self::Item> {
        HashMap::get_mut(self, index)
    }
}
