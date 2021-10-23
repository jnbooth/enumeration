#[cfg(test)]
#[macro_use]
extern crate enumeration_derive;

use std::fmt::Debug;
use std::hash::Hash;
use std::iter::{ExactSizeIterator, FusedIterator, Iterator};
use std::num::*;
use std::ops::{Bound, RangeBounds};

#[macro_use]
mod set;
pub use set::{EnumSet, Wordlike};
mod map;
pub use map::EnumMap;

mod external_trait_impls;

pub trait Enum: Copy + Ord {
    /// Bitwise representation of the type.
    type Rep;

    /// Total number of values in the type.
    const SIZE: usize;

    /// Smallest value in the type.
    ///
    /// Rules: for all `x`, `x.succ() != Some(Self::MIN)`.
    const MIN: Self;

    /// Largest value in the type.
    ///
    /// Rule: for all `x`, `x.pred() != Some(Self::MAX)`.
    const MAX: Self;

    /// Returns `self`'s successor, or `None` if `self == Self::MAX`.
    ///
    /// Rule: for all `x`, `(x == Self::MAX) == x.succ().is_none()`.
    fn succ(self) -> Option<Self>;

    /// Returns `self`'s predecessor, or `None` if `self == Self::MIN`.
    ///
    /// Rule: for all `x`, `(x == Self::MIN) == x.pred().is_none()`.
    fn pred(self) -> Option<Self>;

    /// Bitwise representation of the value.
    fn bit(self) -> Self::Rep;

    /// The value's position in a complete enumeration of the type.
    fn index(self) -> usize;

    /// Inverse of `index`. Returns `None` if out of range.
    fn from_index(i: usize) -> Option<Self> {
        Self::enumerate(..).find(|e| e.index() == i)
    }

    fn enumerate<R: RangeBounds<Self>>(range: R) -> Enumeration<Self> {
        fn invalid_enum<E: Enum>() -> Enumeration<E> {
            Enumeration {
                start: E::MIN,
                end: E::MIN,
                finished: true,
            }
        }
        let start = match range.start_bound() {
            Bound::Unbounded => Self::MIN,
            Bound::Included(&t) => t,
            Bound::Excluded(&t) => match t.succ() {
                Some(succ) => succ,
                None => return invalid_enum(),
            },
        };
        let end = match range.end_bound() {
            Bound::Unbounded => Self::MAX,
            Bound::Included(&t) => t,
            Bound::Excluded(&t) => match t.pred() {
                Some(pred) => pred,
                None => return invalid_enum(),
            },
        };
        if start.index() > end.index() {
            return invalid_enum();
        }
        Enumeration {
            start,
            end,
            finished: false,
        }
    }
}

impl Enum for bool {
    type Rep = u8;
    const SIZE: usize = 2;
    const MIN: Self = false;
    const MAX: Self = true;
    fn succ(self) -> Option<Self> {
        if self {
            None
        } else {
            Some(true)
        }
    }
    fn pred(self) -> Option<Self> {
        if self {
            Some(false)
        } else {
            None
        }
    }
    fn bit(self) -> Self::Rep {
        1 << (self as u8)
    }
    fn index(self) -> usize {
        self as usize
    }
    fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

pub trait Expand: Sized {
    type Expanded: From<Self>;
}
macro_rules! impl_expand {
    ($t:ty, $ex:ty) => {
        impl Expand for $t {
            type Expanded = $ex;
        }
    };
}
impl_expand!(u8, u16);
impl_expand!(i8, i16);
impl_expand!(u16, u32);
impl_expand!(i16, i32);
impl_expand!(u32, u64);
impl_expand!(i32, i64);
impl_expand!(u64, u128);
impl_expand!(i64, i128);
impl_expand!(NonZeroU8, u8);
impl_expand!(NonZeroI8, i8);
impl_expand!(NonZeroU16, u16);
impl_expand!(NonZeroI16, i16);
impl_expand!(NonZeroU32, u32);
impl_expand!(NonZeroI32, i32);
impl_expand!(NonZeroU64, u64);
impl_expand!(NonZeroI64, i64);
impl_expand!(NonZeroU128, u128);
impl_expand!(NonZeroI128, i128);
impl_expand!(NonZeroUsize, usize);
impl_expand!(NonZeroIsize, isize);

impl<E: Enum> Enum for Option<E>
where
    E::Rep: Expand + Wordlike,
{
    type Rep = <E::Rep as Expand>::Expanded;

    const SIZE: usize = E::SIZE + 1;

    const MIN: Self = None;

    const MAX: Self = Some(E::MAX);

    fn succ(self) -> Option<Self> {
        match self {
            None => Some(Some(E::MIN)),
            Some(e) => e.succ().map(Some),
        }
    }

    fn pred(self) -> Option<Self> {
        self.map(|e| e.pred())
    }

    fn bit(self) -> Self::Rep {
        match self {
            None => E::MIN.bit(),
            Some(e) => e.bit().incr(),
        }
        .into()
    }

    fn index(self) -> usize {
        match self {
            None => 0,
            Some(e) => e.index() + 1,
        }
    }

    fn from_index(i: usize) -> Option<Self> {
        if i == 0 {
            Some(None)
        } else {
            E::from_index(i - 1).map(Some)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enumeration<T> {
    finished: bool,
    start: T,
    end: T,
}

impl<T: Enum> Iterator for Enumeration<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else if self.start == self.end {
            self.finished = true;
            Some(self.start)
        } else {
            let at = self.start;
            self.start = at
                .succ()
                .expect("got None from calling Enum::succ() where < Enum::MAX");
            Some(at)
        }
    }

    fn count(self) -> usize {
        if self.finished {
            0
        } else {
            self.end.index() + 1 - self.start.index()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.count();
        (exact, Some(exact))
    }
}
impl<T: Enum> DoubleEndedIterator for Enumeration<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else if self.start == self.end {
            self.finished = true;
            Some(self.end)
        } else {
            let at = self.end;
            self.end = at
                .pred()
                .expect("got None from calling Enum::pred() where > Enum::MIN");
            Some(at)
        }
    }
}
impl<T: Enum> FusedIterator for Enumeration<T> {}
impl<T: Enum> ExactSizeIterator for Enumeration<T> {
    fn len(&self) -> usize {
        self.count()
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[rustfmt::skip] #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
    pub enum DemoEnum { A, B, C, D, E, F, G, H, I, J }

    // Enum tests

    // static assertion proving derive(Enum) picks the smallest possible bitwise representation
    const _: [(); mem::size_of::<<DemoEnum as Enum>::Rep>()] =
        [(); DemoEnum::SIZE / 8 + (DemoEnum::SIZE % 8 != 0) as usize];

    pub fn assert_eqs<T: Eq + Debug, X: Iterator<Item = T>, Y: Iterator<Item = T>>(x: X, y: Y) {
        assert_eq!(x.collect::<Vec<_>>(), y.collect::<Vec<_>>());
    }

    pub fn assert_all<E: Enum + Debug, F: FnMut(E) -> bool>(mut f: F) {
        assert_eqs(
            E::enumerate(..).map(|x| (x, f(x))),
            E::enumerate(..).map(|x| (x, true)),
        );
    }

    #[test]
    fn test_min() {
        assert_all(|x: DemoEnum| x.succ() != Some(DemoEnum::MIN));
    }

    #[test]
    fn test_max() {
        assert_all(|x: DemoEnum| x.pred() != Some(DemoEnum::MAX));
    }

    #[test]
    fn test_succ() {
        assert_all(|x: DemoEnum| (x == DemoEnum::MAX) == x.succ().is_none());
    }

    #[test]
    fn test_pred() {
        assert_all(|x: DemoEnum| (x == DemoEnum::MIN) == x.pred().is_none());
    }

    #[test]
    fn test_index() {
        assert_eqs(DemoEnum::enumerate(..).map(Enum::index), 0..DemoEnum::SIZE);
    }

    #[test]
    fn test_from_index() {
        assert_eqs(
            DemoEnum::enumerate(..).map(Some),
            (0..DemoEnum::SIZE).map(DemoEnum::from_index),
        );
    }

    #[test]
    fn test_from_index_out_of_range() {
        assert_eq!(DemoEnum::from_index(DemoEnum::SIZE), None);
    }

    #[test]
    fn test_count() {
        for x in DemoEnum::enumerate(..) {
            for y in DemoEnum::enumerate(..) {
                let our_count = DemoEnum::enumerate(x..=y).count();
                let std_count = DemoEnum::enumerate(x..=y).fold(0, |count, _| count + 1);
                assert_eq!(
                    our_count,
                    std_count,
                    "for {}..={}, {} != {}",
                    x.to_str(),
                    y.to_str(),
                    our_count,
                    std_count
                );
            }
        }
    }

    #[test]
    fn test_rev() {
        let forward: Vec<_> = DemoEnum::enumerate(..).collect();
        let mut backward: Vec<_> = DemoEnum::enumerate(..).rev().collect();
        backward.reverse();
        assert_eq!(forward, backward);
    }
}
