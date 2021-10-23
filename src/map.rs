use std::hash::Hash;
use std::iter::{FilterMap, Iterator, Zip};
use std::marker::PhantomData;
use std::{slice, vec};

use super::{Enum, Enumeration};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumMap<K, V> {
    inner: Vec<Option<V>>,
    marker: PhantomData<K>,
}

impl<K: Enum, V> Default for EnumMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Enum, V> EnumMap<K, V> {
    pub fn new() -> Self {
        let mut inner = Vec::with_capacity(K::SIZE);
        inner.resize_with(K::SIZE, Default::default);
        Self {
            inner,
            marker: PhantomData,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner[k.index()].replace(v)
    }

    pub fn remove(&mut self, k: K) -> Option<V> {
        self.inner[k.index()].take()
    }
    pub fn get(&self, k: K) -> Option<&V> {
        self.inner[k.index()].as_ref()
    }
    pub fn get_mut(&mut self, k: K) -> Option<&mut V> {
        self.inner[k.index()].as_mut()
    }
}

impl<K: Enum + Copy + Ord, V> EnumMap<K, V> {
    pub fn keys(&self) -> impl '_ + Iterator<Item = K> {
        K::enumerate(..).filter(move |x| self.inner[x.index()].is_some())
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().filter_map(Option::as_ref)
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.into_iter()
    }

    pub fn iter_copied(&self) -> impl '_ + Iterator<Item = (K, V)>
    where
        V: Copy,
    {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(x, m_y)| m_y.map(|y| (x, y)))
    }
}

impl<K: Enum + Copy + Ord, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, vec::IntoIter<Option<V>>>,
        fn((K, Option<V>)) -> Option<(K, V)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(self.inner)
            .filter_map(|(x, m_y)| m_y.map(|y| (x, y)))
    }
}

impl<'a, K: Enum + Copy + Ord, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, slice::Iter<'a, Option<V>>>,
        fn((K, &Option<V>)) -> Option<(K, &V)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(x, m_y)| m_y.as_ref().map(|y| (x, y)))
    }
}

impl<'a, K: Enum + Copy + Ord, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, slice::IterMut<'a, Option<V>>>,
        fn((K, &mut Option<V>)) -> Option<(K, &mut V)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(&mut self.inner)
            .filter_map(|(x, m_y)| m_y.as_mut().map(|y| (x, y)))
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use super::super::tests::*;
}
