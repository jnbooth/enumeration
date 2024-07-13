use std::iter::{FusedIterator, Iterator, Zip};

use crate::enum_trait::{Enum, Enumeration};

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
pub struct Iter<K, V, I: Iterator> {
    inner: Zip<Enumeration<K>, I>,
    f: fn(I::Item) -> Option<V>,
    remaining: usize,
}

impl<K: Enum, V, I: Iterator> Iter<K, V, I> {
    #[inline]
    pub(super) fn new<It>(iter: It, size: usize, f: fn(I::Item) -> Option<V>) -> Self
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

impl<K: Enum, V, I: Iterator> Iterator for Iter<K, V, I> {
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

impl<K: Enum, V, I: Iterator> ExactSizeIterator for Iter<K, V, I> {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<K: Enum, V, I: DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Iter<K, V, I> {
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

impl<K: Enum, V, I: FusedIterator> FusedIterator for Iter<K, V, I> {}
