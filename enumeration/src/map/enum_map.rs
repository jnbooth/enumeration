use std::hash::Hash;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::{slice, vec};

use super::entry::{Entry, OccupiedEntry, VacantEntry};
use super::iter::Iter;
use crate::enum_trait::Enum;

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
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.inner.iter_mut().filter_map(Option::as_mut)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.inner.into_iter().filter_map(std::convert::identity)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter(&self) -> Iter<K, &V, slice::Iter<Option<V>>> {
        self.into_iter()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_mut(&mut self) -> Iter<K, &mut V, slice::IterMut<Option<V>>> {
        self.into_iter()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_copied(&self) -> Iter<K, V, slice::Iter<Option<V>>>
    where
        V: Copy,
    {
        Iter::new(&self.inner, self.size, |v| *v)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(K, &mut V) -> bool,
    {
        for (k, m_v) in K::enumerate(..).zip(&mut self.inner) {
            let erase = match m_v {
                Some(v) => !f(k, v),
                None => false,
            };
            if erase {
                self.size -= 1;
                m_v.take();
            }
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn clear(&mut self) {
        self.size = 0;
        self.inner.fill_with(Default::default);
    }

    #[cfg_attr(feature = "inline-more", inline)]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        let entry = &mut self.inner[key.index()];
        if entry.is_some() {
            Entry::Occupied(OccupiedEntry {
                key,
                value: entry,
                size: &mut self.size,
            })
        } else {
            Entry::Vacant(VacantEntry {
                key,
                value: entry,
                size: &mut self.size,
            })
        }
    }
}

impl<K: Enum, V> Index<K> for EnumMap<K, V> {
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index(&self, key: K) -> &Self::Output {
        self.get(key).expect("no entry found for key")
    }
}

impl<K: Enum, V> IndexMut<K> for EnumMap<K, V> {
    /// Returns a mutable reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key).expect("no entry found for key")
    }
}

impl<K: Enum, V> IntoIterator for EnumMap<K, V> {
    type Item = (K, V);
    type IntoIter = Iter<K, V, vec::IntoIter<Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.inner, self.size, std::convert::identity)
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a EnumMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<K, &'a V, slice::Iter<'a, Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(&self.inner, self.size, Option::as_ref)
    }
}

impl<'a, K: Enum, V> IntoIterator for &'a mut EnumMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = Iter<K, &'a mut V, slice::IterMut<'a, Option<V>>>;

    #[cfg_attr(feature = "inline-more", inline)]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(&mut self.inner, self.size, Option::as_mut)
    }
}
