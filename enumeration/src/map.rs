use std::hash::Hash;
use std::iter::{FilterMap, Iterator, Zip};
use std::marker::PhantomData;
use std::{slice, vec};

use crate::enum_trait::{Enum, Enumeration};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumMap<K, V> {
    inner: Vec<Option<V>>,
    marker: PhantomData<K>,
}

impl<K: Enum, V> Default for EnumMap<K, V> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Enum, V> EnumMap<K, V> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            marker: PhantomData,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        K::SIZE
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.inner.is_empty() {
            self.inner.resize_with(K::SIZE, Default::default);
        }
        self.inner[k.index()].replace(v)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(&mut self, k: K) -> Option<V> {
        self.inner[k.index()].take()
    }

    #[inline]
    pub fn get(&self, k: K) -> Option<&V> {
        self.inner[k.index()].as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self, k: K) -> Option<&mut V> {
        self.inner[k.index()].as_mut()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn keys(&self) -> impl '_ + Iterator<Item = K> {
        K::enumerate(..).filter(move |x| self.inner[x.index()].is_some())
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().filter_map(Option::as_ref)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.into_iter()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_copied(&self) -> impl '_ + Iterator<Item = (K, V)>
    where
        V: Copy,
    {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(x, m_y)| m_y.map(|y| (x, y)))
    }
}

impl<K: Enum, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, vec::IntoIter<Option<V>>>,
        fn((K, Option<V>)) -> Option<(K, V)>,
    >;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(self.inner)
            .filter_map(|(x, m_y)| m_y.map(|y| (x, y)))
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, slice::Iter<'a, Option<V>>>,
        fn((K, &Option<V>)) -> Option<(K, &V)>,
    >;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(x, m_y)| m_y.as_ref().map(|y| (x, y)))
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    #[allow(clippy::type_complexity)]
    type IntoIter = FilterMap<
        Zip<Enumeration<K>, slice::IterMut<'a, Option<V>>>,
        fn((K, &mut Option<V>)) -> Option<(K, &mut V)>,
    >;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        K::enumerate(..)
            .zip(&mut self.inner)
            .filter_map(|(x, m_y)| m_y.as_mut().map(|y| (x, y)))
    }
}
