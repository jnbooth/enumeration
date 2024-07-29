use std::iter::{ExactSizeIterator, FusedIterator, Iterator};

use super::enum_set::EnumSet;
use crate::enumerate::{Enum, Enumeration};

fn enum_fold<T: Enum, B, F>(set: EnumSet<T>, mut fold: F) -> impl FnMut(B, T) -> B
where
    F: FnMut(B, T) -> B,
{
    move |acc, item| {
        if set.contains(item) {
            fold(acc, item)
        } else {
            acc
        }
    }
}

pub struct Iter<T: Enum> {
    set: EnumSet<T>,
    iter: Enumeration<T>,
    remaining: usize,
}

impl<T: Enum> Iter<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    pub(super) fn new(set: EnumSet<T>) -> Self {
        Self {
            iter: T::enumerate(..),
            remaining: set.len(),
            set,
        }
    }
}

impl<T: Enum> Clone for Iter<T> {
    fn clone(&self) -> Self {
        Self {
            set: self.set,
            iter: self.iter,
            remaining: self.remaining,
        }
    }
}

impl<T: Enum> Iterator for Iter<T> {
    type Item = T;

    #[cfg_attr(feature = "inline-more", inline)]
    fn next(&mut self) -> Option<Self::Item> {
        let set = self.set;
        let next = self.iter.find(move |&x| set.contains(x));
        if next.is_some() {
            self.remaining -= 1;
        }
        next
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn count(self) -> usize {
        self.remaining
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn fold<B, F>(self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.fold(init, enum_fold(self.set, fold))
    }
}

impl<T: Enum> ExactSizeIterator for Iter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<T: Enum> DoubleEndedIterator for Iter<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn next_back(&mut self) -> Option<Self::Item> {
        let set = self.set;
        let next = self.iter.rfind(move |&x| set.contains(x));
        if next.is_some() {
            self.remaining -= 1;
        }
        next
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn rfold<B, F>(self, init: B, fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        self.iter.rfold(init, enum_fold(self.set, fold))
    }
}

impl<T: Enum> FusedIterator for Iter<T> {}
