use std::cmp::{min, Ordering};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::{ExactSizeIterator, FromIterator, FusedIterator, Iterator};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::enum_trait::{Enum, Enumeration};
use crate::wordlike::Wordlike;

#[repr(transparent)]
pub struct EnumSet<T: Enum> {
    raw: T::Rep,
}

impl<T: Enum> EnumSet<T>
where
    T::Rep: Wordlike,
{
    pub const fn new() -> Self {
        Self {
            raw: Wordlike::ZERO,
        }
    }

    pub fn clear(&mut self) {
        self.raw = Wordlike::ZERO;
    }

    pub fn insert(&mut self, x: T)
    where
        T::Rep: BitOrAssign,
    {
        self.raw |= x.bit()
    }

    pub fn remove(&mut self, x: T)
    where
        T::Rep: BitAndAssign + Not<Output = T::Rep>,
    {
        self.raw &= !x.bit()
    }

    pub fn contains(&self, x: T) -> bool
    where
        T::Rep: BitAnd<Output = T::Rep> + Eq + Copy,
    {
        self.raw & x.bit() != Wordlike::ZERO
    }
}

impl<T: Enum> Copy for EnumSet<T> where T::Rep: Copy {}

impl<T: Enum> Clone for EnumSet<T>
where
    T::Rep: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Enum> PartialEq for EnumSet<T>
where
    T::Rep: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T: Enum> Eq for EnumSet<T> where T::Rep: Eq {}

impl<T: Enum> PartialOrd for EnumSet<T>
where
    T::Rep: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<T: Enum> Ord for EnumSet<T>
where
    T::Rep: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T: Enum> Hash for EnumSet<T>
where
    T::Rep: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: Enum> Not for EnumSet<T>
where
    T::Rep: Not<Output = T::Rep>,
{
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { raw: !self.raw }
    }
}

macro_rules! bitop {
    ($t:tt, $f:ident) => {
        impl<T: Enum> $t for EnumSet<T>
        where
            T::Rep: $t<Output = T::Rep>,
        {
            type Output = Self;
            fn $f(self, other: Self) -> Self::Output {
                Self {
                    raw: self.raw.$f(other.raw),
                }
            }
        }
        impl<T: Enum> $t<T> for EnumSet<T>
        where
            T::Rep: $t<Output = T::Rep>,
        {
            type Output = Self;
            fn $f(self, other: T) -> Self::Output {
                Self {
                    raw: self.raw.$f(other.bit()),
                }
            }
        }
    };
}
macro_rules! bitassign {
    ($t:tt, $f:ident) => {
        impl<T: Enum> $t for EnumSet<T>
        where
            T::Rep: $t,
        {
            fn $f(&mut self, other: Self) {
                self.raw.$f(other.raw)
            }
        }
        impl<T: Enum> $t<T> for EnumSet<T>
        where
            T::Rep: $t,
        {
            fn $f(&mut self, other: T) {
                self.raw.$f(other.bit())
            }
        }
    };
}
bitop!(BitAnd, bitand);
bitassign!(BitAndAssign, bitand_assign);
bitop!(BitOr, bitor);
bitassign!(BitOrAssign, bitor_assign);
bitop!(BitXor, bitxor);
bitassign!(BitXorAssign, bitxor_assign);

impl<T: Enum> FromIterator<T> for EnumSet<T>
where
    T::Rep: BitOr<Output = T::Rep> + Wordlike,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            raw: iter
                .into_iter()
                .map(T::bit)
                .fold(Wordlike::ZERO, BitOr::bitor),
        }
    }
}

impl<'a, T: Enum> FromIterator<&'a T> for EnumSet<T>
where
    T::Rep: BitOr<Output = T::Rep> + Wordlike,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        Self {
            raw: iter
                .into_iter()
                .map(|&x| T::bit(x))
                .fold(Wordlike::ZERO, BitOr::bitor),
        }
    }
}

impl<T: Enum> Default for EnumSet<T>
where
    T::Rep: Wordlike,
{
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub mod __private {
    use super::*;

    pub const fn construct_set<T: Enum>(raw: T::Rep, _type_holder: T) -> EnumSet<T> {
        EnumSet { raw }
    }
}

#[macro_export]
macro_rules! enums {
    () => ($crate::EnumSet::new());
    ($i1:expr $(,)?) => ({
        #[allow(unused_imports)]
        use $crate::Enum;
        $crate::__private::construct_set($i1.bit(), $i1)
    });
    ($i1:expr, $($i:expr),+ $(,)?) => ({
        #[cfg(debug_assertions)]
        let _ = [$i1, $($i),+]; // all items are same type
        #[allow(unused_imports)]
        use $crate::Enum;
        $crate::__private::construct_set($i1.bit()$(|$i.bit())*, $i1)
    });
}

impl<T: Enum> IntoIterator for EnumSet<T>
where
    T::Rep: BitAnd<Output = T::Rep> + Wordlike + Eq + Copy,
{
    type Item = T;
    type IntoIter = EnumIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        EnumIter {
            set: self,
            iter: T::enumerate(..),
        }
    }
}

impl<T: Enum> Debug for EnumSet<T>
where
    T: Debug,
    EnumSet<T>: IntoIterator<Item = T> + Copy,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

pub struct EnumIter<T: Enum> {
    set: EnumSet<T>,
    iter: Enumeration<T>,
}

impl<T: Enum> Iterator for EnumIter<T>
where
    T::Rep: BitAnd<Output = T::Rep> + Wordlike + Eq + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let set = self.set;
        self.iter.find(move |&x| set.contains(x))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = Wordlike::count_ones(self.set.raw) as usize;
        (0, Some(min(self.iter.len(), count)))
    }

    fn count(self) -> usize {
        let set = self.set;
        self.iter.map(move |x| set.contains(x) as usize).sum()
    }

    fn fold<Acc, Fold>(self, init: Acc, mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let set = self.set;
        self.iter.fold(init, move |acc, item| {
            if set.contains(item) {
                fold(acc, item)
            } else {
                acc
            }
        })
    }
}

impl<T: Enum> FusedIterator for EnumIter<T> where EnumIter<T>: Iterator<Item = T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip] #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
    pub enum DemoEnum { A, B, C, D, E, F, G, H, I, J }

    // EnumSet tests

    #[test]
    fn test_enumerate() {
        let _: EnumSet<DemoEnum> = enums![DemoEnum::A, DemoEnum::C];
        assert_eq!(
            EnumSet { raw: !0 }.into_iter().collect::<Vec<DemoEnum>>(),
            Enum::enumerate(..).collect::<Vec<DemoEnum>>()
        );
    }
}
