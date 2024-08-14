use std::iter::{FusedIterator, Iterator, Zip};
use std::slice;

use crate::enumerate::{Enum, Enumeration};

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
        for (k, v) in &mut self.inner {
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

#[inline]
fn matches_mut<K: Copy, V, P>(key: K, val: &mut Option<V>, pred: &mut P) -> bool
where
    P: FnMut(K, &mut V) -> bool,
{
    let Some(val) = val.as_mut() else {
        return false;
    };
    pred(key, val)
}

fn drain_fold<'a, B, K: Copy, V: 'a>(
    mut pred: impl FnMut(K, &mut V) -> bool,
    mut fold: impl FnMut(B, (K, V)) -> B,
    size: &'a mut usize,
) -> impl FnMut(B, (K, &'a mut Option<V>)) -> B {
    move |acc, (k, item)| {
        if matches_mut(k, item, &mut pred) {
            *size -= 1;
            fold(acc, (k, item.take().unwrap()))
        } else {
            acc
        }
    }
}

impl<K: Enum, V, I: FusedIterator> FusedIterator for Iter<K, V, I> {}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ExtractIf<'a, K, V, P> {
    inner: Zip<Enumeration<K>, slice::IterMut<'a, Option<V>>>,
    pred: P,
    size: &'a mut usize,
}

impl<'a, K: Enum, V, P: FnMut(K, &mut V) -> bool> ExtractIf<'a, K, V, P> {
    #[inline]
    pub(super) fn new(iter: slice::IterMut<'a, Option<V>>, size: &'a mut usize, pred: P) -> Self {
        Self {
            inner: K::enumerate(..).zip(iter),
            pred,
            size,
        }
    }
}

impl<'a, K: Enum, V, P: FnMut(K, &mut V) -> bool> Iterator for ExtractIf<'a, K, V, P> {
    type Item = (K, V);

    #[cfg_attr(feature = "inline-more", inline)]
    fn next(&mut self) -> Option<Self::Item> {
        for (k, v) in &mut self.inner {
            if matches_mut(k, v, &mut self.pred) {
                *self.size -= 1;
                return Some((k, v.take().unwrap()));
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(*self.size))
    }

    #[inline]
    fn count(mut self) -> usize {
        let mut count = 0;
        for (k, v) in &mut self.inner {
            if matches_mut(k, v, &mut self.pred) {
                *v = None;
                *self.size -= 1;
                count += 1;
            }
        }
        count
    }

    #[inline]
    fn fold<B, F>(self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner
            .fold(init, drain_fold(self.pred, fold, self.size))
    }
}

impl<'a, K: Enum, V, P: FnMut(K, &mut V) -> bool> ExactSizeIterator for ExtractIf<'a, K, V, P> {
    #[inline]
    fn len(&self) -> usize {
        *self.size
    }
}

impl<'a, K: Enum, V, P: FnMut(K, &mut V) -> bool> DoubleEndedIterator for ExtractIf<'a, K, V, P> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some((k, v)) = self.inner.next_back() {
            if matches_mut(k, v, &mut self.pred) {
                *self.size -= 1;
                return Some((k, v.take().unwrap()));
            }
        }
        None
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn rfold<B, F>(mut self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner
            .rfold(init, drain_fold(&mut self.pred, fold, self.size))
    }
}

impl<'a, K: Enum, V, P: FnMut(K, &mut V) -> bool> FusedIterator for ExtractIf<'a, K, V, P> {}
