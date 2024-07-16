use std::fmt::Debug;
use std::hash::Hash;
use std::iter::{ExactSizeIterator, FusedIterator, Iterator};

use super::enum_trait::Enum;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Enumeration<T> {
    pub(super) finished: bool,
    pub(super) start: T,
    pub(super) end: T,
}

impl<T: Enum> Iterator for Enumeration<T> {
    type Item = T;

    #[cfg_attr(feature = "inline-more", inline)]
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

    #[cfg_attr(feature = "inline-more", inline)]
    fn fold<B, F>(self, init: B, mut fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if self.finished {
            return init;
        }
        let mut accum = init;
        let mut val = self.start;
        loop {
            accum = fold(accum, val);
            if val == self.end {
                return accum;
            }
            val = match val.succ() {
                None => return accum,
                Some(val) => val,
            }
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn count(self) -> usize {
        if self.finished {
            0
        } else {
            self.end.index() + 1 - self.start.index()
        }
    }

    #[cfg_attr(feature = "inline-more", inline)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.count();
        (exact, Some(exact))
    }
}
impl<T: Enum> DoubleEndedIterator for Enumeration<T> {
    #[cfg_attr(feature = "inline-more", inline)]
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

    #[cfg_attr(feature = "inline-more", inline)]
    fn rfold<B, F>(self, init: B, mut fold: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if self.finished {
            return init;
        }
        let mut accum = init;
        let mut val = self.end;
        loop {
            accum = fold(accum, val);
            if val == self.start {
                return accum;
            }
            val = match val.pred() {
                None => return accum,
                Some(val) => val,
            }
        }
    }
}
impl<T: Enum> FusedIterator for Enumeration<T> {}
impl<T: Enum> ExactSizeIterator for Enumeration<T> {
    #[inline]
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
