use std::cmp::Ordering;
use std::iter::Iterator;
use std::ops::{Bound, RangeBounds};

use super::iter::Enumeration;
use crate::Wordlike;

pub trait Enum: Copy + Ord {
    /// Bitwise representation of the type.
    type Rep: Wordlike;

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
    #[cfg_attr(feature = "inline-more", inline)]
    fn from_index(i: usize) -> Option<Self> {
        Self::enumerate(..).find(|e| e.index() == i)
    }

    fn enumerate<R: RangeBounds<Self>>(range: R) -> Enumeration<Self> {
        fn invalid_enum<T: Enum>() -> Enumeration<T> {
            Enumeration {
                start: T::MIN,
                end: T::MIN,
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

impl<T: Enum> Enum for Option<T> {
    type Rep = T::Rep;

    const SIZE: usize = T::SIZE + 1;

    const MIN: Self = None;

    const MAX: Self = Some(T::MAX);

    #[cfg_attr(feature = "inline-more", inline)]
    fn succ(self) -> Option<Self> {
        match self {
            None => Some(Some(T::MIN)),
            Some(e) => e.succ().map(Some),
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn pred(self) -> Option<Self> {
        match self {
            None => None,
            Some(e) => Some(e.pred()),
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn bit(self) -> Self::Rep {
        match self {
            None => T::MIN.bit(),
            Some(e) => e.bit().incr(),
        }
        .into()
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn index(self) -> usize {
        match self {
            None => 0,
            Some(e) => e.index() + 1,
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn from_index(i: usize) -> Option<Self> {
        if i == 0 {
            Some(None)
        } else {
            T::from_index(i - 1).map(Some)
        }
    }
}

impl Enum for bool {
    type Rep = u8;
    const SIZE: usize = 2;
    const MIN: Self = false;
    const MAX: Self = true;

    #[cfg_attr(feature = "inline-more", inline)]
    fn succ(self) -> Option<Self> {
        if self {
            None
        } else {
            Some(true)
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn pred(self) -> Option<Self> {
        if self {
            Some(false)
        } else {
            None
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn bit(self) -> Self::Rep {
        1 << (self as u8)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn index(self) -> usize {
        self as usize
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

impl Enum for Ordering {
    type Rep = u8;

    const SIZE: usize = 3;

    const MIN: Self = Ordering::Less;

    const MAX: Self = Ordering::Greater;

    #[cfg_attr(feature = "inline-more", inline)]
    fn succ(self) -> Option<Self> {
        match self {
            Ordering::Less => Some(Ordering::Equal),
            Ordering::Equal => Some(Ordering::Greater),
            Ordering::Greater => None,
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn pred(self) -> Option<Self> {
        match self {
            Ordering::Less => None,
            Ordering::Equal => Some(Ordering::Less),
            Ordering::Greater => Some(Ordering::Equal),
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn bit(self) -> Self::Rep {
        1 << (self as i8 + 1)
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn index(self) -> usize {
        (self as i8 + 1) as usize
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn from_index(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Less),
            1 => Some(Self::Equal),
            2 => Some(Self::Greater),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::mem;

    use super::*;

    #[rustfmt::skip] #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    enum DemoEnum { A, B, C, D, E, F, G, H, I, J }

    // Enum tests

    // static assertion proving derive(Enum) picks the smallest possible bitwise representation
    const _: [(); mem::size_of::<<DemoEnum as Enum>::Rep>()] =
        [(); DemoEnum::SIZE / 8 + (DemoEnum::SIZE % 8 != 0) as usize];

    fn assert_eqs<T: Eq + Debug, X: Iterator<Item = T>, Y: Iterator<Item = T>>(x: X, y: Y) {
        assert_eq!(x.collect::<Vec<_>>(), y.collect::<Vec<_>>());
    }

    fn assert_all<E: Enum + Debug, F: FnMut(E) -> bool>(mut f: F) {
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
                    our_count, std_count,
                    "for {:?}..={:?}, {} != {}",
                    x, y, our_count, std_count
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
