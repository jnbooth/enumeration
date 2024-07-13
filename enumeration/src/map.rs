use std::hash::Hash;
use std::iter::{FusedIterator, Iterator, Zip};
use std::marker::PhantomData;
use std::{slice, vec};

use crate::enum_trait::{Enum, Enumeration};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnumMap<K, V> {
    inner: Vec<Option<V>>,
    size: usize,
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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            size: 0,
            marker: PhantomData,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        K::SIZE
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        if self.inner.is_empty() {
            self.inner.resize_with(K::SIZE, Default::default);
        }
        let old_val = self.inner[k.index()].replace(v);
        if old_val.is_none() {
            self.size += 1;
        }
        old_val
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(&mut self, k: K) -> Option<V> {
        let old_val = self.inner[k.index()].take();
        if old_val.is_some() {
            self.size -= 1;
        }
        old_val
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
    pub fn contains_key(&self, k: K) -> bool {
        self.inner[k.index()].is_some()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn keys(&self) -> impl '_ + Iterator<Item = K> {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(k, v)| match v {
                Some(_) => Some(k),
                None => None,
            })
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().filter_map(Option::as_ref)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.inner.into_iter().filter_map(std::convert::identity)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter(&self) -> MapIter<K, &V, slice::Iter<Option<V>>> {
        self.into_iter()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_copied(&self) -> MapIter<K, V, slice::Iter<Option<V>>>
    where
        V: Copy,
    {
        MapIter::new(&self.inner, self.size, |v| *v)
    }
}

fn map_fold<B, K, From, To>(
    mut f: impl FnMut(From) -> Option<To>,
    mut fold: impl FnMut(B, (K, To)) -> B,
) -> impl FnMut(B, (K, From)) -> B {
    move |acc, (k, item)| match f(item) {
        Some(x) => fold(acc, (k, x)),
        None => acc,
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MapIter<K, V, I: Iterator> {
    inner: Zip<Enumeration<K>, I>,
    f: fn(I::Item) -> Option<V>,
    remaining: usize,
}

impl<K: Enum, V, I: Iterator> MapIter<K, V, I> {
    #[inline]
    fn new<It>(iter: It, size: usize, f: fn(I::Item) -> Option<V>) -> Self
    where
        It: IntoIterator<IntoIter = I>,
    {
        Self {
            inner: K::enumerate(..).zip(iter.into_iter()),
            f,
            remaining: size,
        }
    }
}

impl<K: Enum, V, I: Iterator> Iterator for MapIter<K, V, I> {
    type Item = (K, V);

    #[cfg_attr(feature = "inline-more", inline)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((k, v)) = self.inner.next() {
            if let Some(item) = (self.f)(v) {
                self.remaining -= 1;
                return Some((k, item));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.remaining
    }

    #[inline]
    fn fold<B, F>(self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, map_fold(self.f, fold))
    }
}

impl<K: Enum, V, I: Iterator> ExactSizeIterator for MapIter<K, V, I> {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<K: Enum, V, I: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator
    for MapIter<K, V, I>
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((k, v)) = self.inner.next_back() {
            if let Some(item) = (self.f)(v) {
                self.remaining -= 1;
                return Some((k, item));
            }
        }
        None
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn rfold<B, F>(self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.rfold(init, map_fold(self.f, fold))
    }
}

impl<K: Enum, V, I: FusedIterator> FusedIterator for MapIter<K, V, I> {}

impl<K: Enum, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    type IntoIter = MapIter<K, V, vec::IntoIter<Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        MapIter::new(self.inner, self.size, std::convert::identity)
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = MapIter<K, &'a V, slice::Iter<'a, Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        MapIter::new(&self.inner, self.size, Option::as_ref)
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = MapIter<K, &'a mut V, slice::IterMut<'a, Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        MapIter::new(&mut self.inner, self.size, Option::as_mut)
    }
}
