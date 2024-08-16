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

    #[allow(clippy::cast_sign_loss)]
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

    use super::*;

    #[rustfmt::skip]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    enum SingleEnum { A }

    #[rustfmt::skip]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    enum DoubleEnum { A, B }

    #[rustfmt::skip] #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    enum ManyEnum { A, B, C, D, E, F, G, H, I, J }

    // Enum tests

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
        fn test<E: Debug + Enum>() {
            assert_all(|e: E| e.succ() != Some(E::MIN));
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_max() {
        fn test<E: Debug + Enum>() {
            assert_all(|e: E| e.pred() != Some(E::MAX));
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_succ() {
        fn test<E: Debug + Enum>() {
            assert_all(|e: E| (e == E::MAX) == e.succ().is_none());
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_pred() {
        fn test<E: Debug + Enum>() {
            assert_all(|e: E| (e == E::MIN) == e.pred().is_none());
            assert_all(|e: E| (e == E::MIN) == e.pred().is_none());
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_index() {
        fn test<E: Debug + Enum>() {
            assert_eqs(E::enumerate(..).map(Enum::index), 0..E::SIZE);
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_from_index() {
        fn test<E: Debug + Enum>() {
            assert_eqs(E::enumerate(..).map(Some), (0..E::SIZE).map(E::from_index));
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_from_index_out_of_range() {
        fn test<E: Debug + Enum>() {
            assert_eq!(E::from_index(E::SIZE), None);
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_count() {
        fn test<E: Debug + Enum>() {
            for x in E::enumerate(..) {
                for y in E::enumerate(..) {
                    let our_count = E::enumerate(x..=y).count();
                    let std_count = E::enumerate(x..=y).fold(0, |count, _| count + 1);
                    assert_eq!(
                        our_count, std_count,
                        "for {x:?}..={y:?}, {our_count} != {std_count}"
                    );
                }
            }
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }

    #[test]
    fn test_rev() {
        fn test<E: Debug + Enum>() {
            let forward: Vec<_> = E::enumerate(..).collect();
            let mut backward: Vec<_> = E::enumerate(..).rev().collect();
            backward.reverse();
            assert_eq!(forward, backward);
        }
        test::<SingleEnum>();
        test::<DoubleEnum>();
        test::<ManyEnum>();
    }
}
