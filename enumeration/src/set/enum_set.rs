use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::{FromIterator, Iterator};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use super::iter::Iter;
use crate::enumerate::Enum;
use crate::wordlike::Wordlike;

#[repr(transparent)]
pub struct EnumSet<T: Enum> {
    raw: T::Rep,
}

impl<T: Enum> EnumSet<T>
where
    T::Rep: Wordlike,
{
    /// Creates an empty `EnumSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let set: EnumSet<TextStyle> = EnumSet::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self { raw: T::Rep::ZERO }
    }

    /// Returns the number of elements the set can hold without reallocating.
    /// This is equivalent to [`T::SIZE`].
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let set: EnumSet<TextStyle> = EnumSet::new();
    /// assert!(set.capacity() == 6);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        T::SIZE
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let mut v: EnumSet<TextStyle> = EnumSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(TextStyle::Italic);
    /// assert_eq!(v.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        T::Rep::count_ones(self.raw) as usize
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let mut v: EnumSet<TextStyle> = EnumSet::new();
    /// assert!(v.is_empty());
    /// v.insert(TextStyle::Italic);
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw == Wordlike::ZERO
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(e)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let mut set = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Highlight];
    /// set.retain(|k| k == TextStyle::Blink || k == TextStyle::Highlight);
    /// assert_eq!(set, enums![TextStyle::Blink, TextStyle::Highlight]);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(T) -> bool,
    {
        for val in T::enumerate(..) {
            let bit = val.bit();
            if ((self.raw & bit) != Wordlike::ZERO) && !f(val) {
                self.raw &= !bit;
            }
        }
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let set: EnumSet<TextStyle> = EnumSet::new();
    ///
    /// let mut v = EnumSet::new();
    /// v.insert(TextStyle::Bold);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.raw = Wordlike::ZERO;
    }

    /// Returns a new set containing the values not contained by this set.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Bold, Italic, Strikeout, Underline }
    ///
    /// let set = enums![TextStyle::Bold, TextStyle::Strikeout];
    /// let inverse = set.inverse();
    /// assert_eq!(inverse, enums![TextStyle::Italic, TextStyle::Underline]);
    /// ```
    #[inline]
    pub fn inverse(&self) -> Self {
        Self {
            raw: !self.raw & T::Rep::mask(T::SIZE as u32),
        }
    }

    /// Returns a new set representing the difference,
    /// i.e., the values that are in `self` but not in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let a = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// let b = enums![TextStyle::Bold, TextStyle::Italic, TextStyle::Underline];
    ///
    /// // Can be seen as `a - b`.
    /// let diff = a.difference(&b);
    /// assert_eq!(diff, enums![TextStyle::Blink]);
    /// ```
    #[inline]
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            raw: (self.raw | other.raw) ^ other.raw,
        }
    }

    /// Returns a new set representing the symmetric difference,
    /// i.e., the values that are in `self` or in `other` but not in both.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let a = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// let b = enums![TextStyle::Bold, TextStyle::Italic, TextStyle::Underline];
    ///
    /// let diff = a.symmetric_difference(&b);
    /// assert_eq!(diff, enums![TextStyle::Blink, TextStyle::Underline]);
    /// ```
    #[inline]
    pub fn symmetric_difference(&self, other: &Self) -> Self {
        Self {
            raw: self.raw ^ other.raw,
        }
    }

    /// Returns a new set representing the intersection,
    /// i.e., the values that are both in `self` and `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let a = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// let b = enums![TextStyle::Bold, TextStyle::Italic, TextStyle::Underline];
    /// let intersection = a.intersection(&b);
    /// assert_eq!(intersection, enums![TextStyle::Bold, TextStyle::Italic]);
    /// ```
    #[inline]
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            raw: (self.raw & other.raw),
        }
    }

    /// Visits the values representing the union,
    /// i.e., all the values in `self` or `other`, without duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let a = enums![TextStyle::Blink, TextStyle::Bold];
    /// let b = enums![TextStyle::Bold, TextStyle::Italic];
    /// let union = a.union(&b);
    /// assert_eq!(union, enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic]);
    /// ```
    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            raw: self.raw | other.raw,
        }
    }

    /// Returns `true` if the set contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let set = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// assert_eq!(set.contains(TextStyle::Bold), true);
    /// assert_eq!(set.contains(TextStyle::Highlight), false);
    /// ```
    #[inline]
    pub fn contains(&self, x: T) -> bool {
        self.raw & x.bit() != Wordlike::ZERO
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let a = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// let mut b = EnumSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(TextStyle::Highlight);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(TextStyle::Blink);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    #[inline]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.raw & other.raw == Wordlike::ZERO
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let sup = enums![TextStyle::Blink, TextStyle::Bold, TextStyle::Italic];
    /// let mut set = EnumSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(TextStyle::Blink);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(TextStyle::Highlight);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.raw | other.raw == other.raw
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let sub = enums![TextStyle::Blink, TextStyle::Bold];
    /// let mut set = EnumSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(TextStyle::Blink);
    /// set.insert(TextStyle::Highlight);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(TextStyle::Bold);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        self.raw | other.raw == self.raw
    }

    /// Adds a value to the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let mut set = enums![TextStyle::Blink];
    /// set.insert(TextStyle::Bold);
    ///
    /// assert_eq!(set, enums![TextStyle::Blink, TextStyle::Bold]);
    /// ```
    #[inline]
    pub fn insert(&mut self, x: T) {
        self.raw |= x.bit();
    }

    /// Removes a value from the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use enumeration::{Enum, EnumSet, enums};
    ///
    /// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
    /// pub enum TextStyle { Blink, Bold, Highlight, Italic, Strikeout, Underline }
    ///
    /// let mut set = EnumSet::new();
    ///
    /// set.insert(TextStyle::Blink);
    /// set.insert(TextStyle::Bold);
    /// set.remove(TextStyle::Bold);
    /// assert_eq!(set, enums![TextStyle::Blink]);
    /// ```
    #[inline]
    pub fn remove(&mut self, x: T) {
        self.raw &= !x.bit();
    }

    /// Returns the underlying bit representation of the enum flags. Intended for FFI.
    #[inline]
    pub const fn from_raw(raw: T::Rep) -> Self {
        Self { raw }
    }

    /// Constructs a set from the underlying bit representation of the enum flags. Intended for FFI.
    #[inline]
    pub const fn to_raw(&self) -> T::Rep {
        self.raw
    }
}

impl<T: Enum> Copy for EnumSet<T> {}

impl<T: Enum> Clone for EnumSet<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Enum> PartialEq for EnumSet<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }

    #[allow(clippy::partialeq_ne_impl)]
    #[inline]
    fn ne(&self, other: &Self) -> bool {
        self.raw != other.raw
    }
}
impl<T: Enum> Eq for EnumSet<T> {}

impl<T: Enum> PartialOrd for EnumSet<T> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.raw.partial_cmp(&other.raw)
    }
}

impl<T: Enum> Ord for EnumSet<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T: Enum> Hash for EnumSet<T>
where
    T::Rep: Hash,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T: Enum> Not for EnumSet<T> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        self.inverse()
    }
}

impl<T: Enum> Default for EnumSet<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Enum> Debug for EnumSet<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

macro_rules! bitop {
    ($t:tt, $f:ident) => {
        impl<T: Enum> $t for EnumSet<T> {
            type Output = Self;

            #[inline]
            fn $f(self, other: Self) -> Self::Output {
                Self {
                    raw: self.raw.$f(other.raw),
                }
            }
        }
        impl<T: Enum> $t<T> for EnumSet<T> {
            type Output = Self;

            #[inline]
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
        impl<T: Enum> $t for EnumSet<T> {
            #[inline]
            fn $f(&mut self, other: Self) {
                self.raw.$f(other.raw)
            }
        }
        impl<T: Enum> $t<T> for EnumSet<T> {
            #[inline]
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

impl<T: Enum> FromIterator<T> for EnumSet<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            raw: iter
                .into_iter()
                .map(T::bit)
                .fold(Wordlike::ZERO, BitOr::bitor),
        }
    }
}

impl<'a, T: Enum> FromIterator<&'a T> for EnumSet<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        Self {
            raw: iter
                .into_iter()
                .map(|&x| T::bit(x))
                .fold(Wordlike::ZERO, BitOr::bitor),
        }
    }
}

impl<T: Enum, const N: usize> From<[T; N]> for EnumSet<T> {
    #[inline]
    fn from(value: [T; N]) -> Self {
        Self::from_iter(value)
    }
}

impl<T: Enum> IntoIterator for EnumSet<T> {
    type Item = T;
    type IntoIter = Iter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<T: Enum> Extend<T> for EnumSet<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<'a, T: Enum> Extend<&'a T> for EnumSet<T> {
    #[cfg_attr(feature = "inline-more", inline)]
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for &item in iter {
            self.insert(item);
        }
    }
}

#[doc(hidden)]
pub mod __private {
    use super::*;

    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
    pub enum DemoEnum { A, B, C, D, E, F, G, H, I, J }

    fn to_vec<T: IntoIterator<Item = DemoEnum>>(set: T) -> Vec<DemoEnum> {
        set.into_iter().collect()
    }

    // EnumSet tests

    #[test]
    fn test_enumerate() {
        let _: EnumSet<DemoEnum> = enums![DemoEnum::A, DemoEnum::C];
        assert_eq!(to_vec(EnumSet { raw: !0 }), to_vec(Enum::enumerate(..)));
    }

    #[test]
    fn test_inverse() {
        let set = enums![
            DemoEnum::A,
            DemoEnum::C,
            DemoEnum::H,
            DemoEnum::I,
            DemoEnum::J
        ];
        let inverse = enums![
            DemoEnum::B,
            DemoEnum::D,
            DemoEnum::E,
            DemoEnum::F,
            DemoEnum::G
        ];
        assert_eq!(to_vec(set.inverse()), to_vec(inverse))
    }
}
