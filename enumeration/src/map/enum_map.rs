use std::hash::Hash;
use std::iter::Iterator;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::{slice, vec};

use super::entry::{Entry, OccupiedEntry, VacantEntry};
use super::iter::{ExtractIf, Iter};
use crate::enumerate::Enum;

/// A lookup map using enumerated types as keys.
///
/// It is required that the keys implement the [`Enum`] trait, although this can frequently be
/// achieved by using `#[derive(Enum)]`.
/// If you implement these yourself, it is important that the following
/// property holds:
///
/// ```text
/// k1 == k2 -> k1.index() == k2.index()
/// ```
///
/// Violating this property is a logic error.
///
/// The backing store is a `Vec<Option<V>>` of size equal to [`K::SIZE`].
///
/// [`Enum`]: crate::Enum
/// [`K::SIZE`]: crate::Enum::SIZE
///
/// # Examples
/// ```
/// use enumeration::{Enum, EnumMap};
///
/// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
/// pub enum Season { Winter, Spring, Summer, Fall }
///
/// // Type inference lets us omit an explicit type signature (which
/// // would be `EnumMap<Season, u32>` in this example).
/// let mut rainfall = EnumMap::new();
///
/// // Record rainfall.
/// rainfall.insert(Season::Winter, 11);
/// rainfall.insert(Season::Spring, 13);
/// rainfall.insert(Season::Summer, 3);
///
/// // Check for a specific one.
/// if !rainfall.contains_key(Season::Fall) {
///     println!("Rainfall has been recorded for {} seasons, but not fall.", rainfall.len());
/// }
///
/// // Delete one.
/// rainfall.remove(Season::Winter);
///
/// // Look up the values associated with some keys.
/// let to_find = [Season::Winter, Season::Spring, Season::Summer];
/// for &season in &to_find {
///     match rainfall.get(season) {
///         Some(amount) => println!("{season:?}: {amount}"),
///         None => println!("{season:?} has no record.")
///     }
/// }
///
/// // Look up the value for a key (will panic if the key is not found).
/// println!("Rainfall for summer: {}", rainfall[Season::Summer]);
///
/// // Iterate over everything.
/// for (season, amount) in &rainfall {
///     println!("{season:?}: \"{amount}\"");
/// }
/// ```
///
/// An `EnumMap` with a known list of items can be initialized from an array:
///
/// ```
/// use enumeration::{Enum, EnumMap};
///
/// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
/// pub enum Season { Winter, Spring, Summer, Fall }
///
/// let rainfall = EnumMap::from([
///     (Season::Spring, 10),
///     (Season::Summer, 3),
///     (Season::Winter, 8),
/// ]);
/// ```
///
/// `EnumMap` implements an [`Entry` API](#method.entry), which allows
/// for complex methods of getting, setting, updating and removing keys and
/// their values:
///
/// ```
/// use enumeration::{Enum, EnumMap};
///
/// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Enum)]
/// pub enum Stat { Health, Defense, Attack, Mana, Stamina, Stagger }
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `EnumMap<Stat, u8>` in this example).
/// let mut player_stats = EnumMap::new();
///
/// fn random_stat_buff() -> u8 {
///     // could actually return some random value here - let's just return
///     // some fixed value for now
///     42
/// }
///
/// // insert a key only if it doesn't already exist
/// player_stats.entry(Stat::Health).or_insert(100);
///
/// // insert a key using a function that provides a new value only if it
/// // doesn't already exist
/// player_stats.entry(Stat::Defense).or_insert_with(random_stat_buff);
///
/// // update a key, guarding against the key possibly not being set
/// let stat = player_stats.entry(Stat::Attack).or_insert(100);
/// *stat += random_stat_buff();
///
/// // modify an entry before an insert with in-place mutation
/// player_stats.entry(Stat::Mana).and_modify(|mana| *mana += 200).or_insert(100);
/// ```
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
    /// Creates an empty `EnumMap`.
    ///
    /// The map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// let mut map: EnumMap<Ordering, i32> = EnumMap::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            size: 0,
            marker: PhantomData,
        }
    }

    /// Returns the number of elements the map can hold.
    /// This is equivalent to [`K::SIZE`].
    ///
    /// [`K::SIZE`]: Enum::SIZE
    #[inline]
    pub const fn capacity(&self) -> usize {
        K::SIZE
    }

    /// An iterator visiting all keys.
    /// The iterator element type is `K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// for key in map.keys() {
    ///     println!("{key:?}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over keys takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn keys(&self) -> impl '_ + Iterator<Item = K> {
        K::enumerate(..)
            .zip(&self.inner)
            .filter_map(|(k, v)| match v {
                Some(_) => Some(k),
                None => None,
            })
    }

    /// An iterator visiting all values.
    /// The iterator element type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// for val in map.values() {
    ///     println!("{val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over values takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().filter_map(Option::as_ref)
    }

    /// An iterator visiting all values mutably.
    /// The iterator element type is `&'a mut V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// for val in map.values_mut() {
    ///     *val = *val + 10;
    /// }
    ///
    /// for val in map.values() {
    ///     println!("{val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over values takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.inner.iter_mut().filter_map(Option::as_mut)
    }

    /// Creates a consuming iterator visiting all the values.
    /// The map cannot be used after calling this.
    /// The iterator element type is `V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// let mut vec: Vec<i32> = map.into_values().collect();
    /// // The `IntoValues` iterator produces values in arbitrary order, so
    /// // the values must be sorted to test them against a sorted array.
    /// vec.sort_unstable();
    /// assert_eq!(vec, [1, 5]);
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over values takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_values(self) -> impl Iterator<Item = V> {
        self.inner.into_iter().flatten()
    }

    /// An iterator visiting all key-value pairs.
    /// The iterator element type is `(K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {key:?} val: {val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over map takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter(&self) -> Iter<K, &V, slice::Iter<Option<V>>> {
        self.into_iter()
    }

    /// An iterator visiting all key-value pairs.
    /// with mutable references to the values.
    /// The iterator element type is `(K, &'a mut V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::from([
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Less, 5),
    /// ]);
    ///
    /// // Update all values
    /// for (_, val) in map.iter_mut() {
    ///     *val *= 2;
    /// }
    ///
    /// for (key, val) in &map {
    ///     println!("key: {key:?} val: {val}");
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, iterating over map takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn iter_mut(&mut self) -> Iter<K, &mut V, slice::IterMut<Option<V>>> {
        self.into_iter()
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut a = EnumMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(Ordering::Less, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.size
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut a = EnumMap::new();
    /// assert!(a.is_empty());
    /// a.insert(Ordering::Less, "a");
    /// assert!(!a.is_empty());
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Clears the map, returning all key-value pairs as an iterator. Keeps the
    /// allocated memory for reuse.
    ///
    /// If the returned iterator is dropped before being fully consumed, it
    /// drops the remaining key-value pairs. The returned iterator keeps a
    /// mutable borrow on the map to optimize its implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut a = EnumMap::new();
    /// a.insert(Ordering::Less, "a");
    /// a.insert(Ordering::Greater, "b");
    ///
    /// for (k, v) in a.drain().take(1) {
    ///     assert!(k == Ordering::Less || k == Ordering::Greater);
    ///     assert!(v == "a" || v == "b");
    /// }
    ///
    /// assert!(a.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn drain(&mut self) -> Iter<K, V, vec::Drain<Option<V>>> {
        let size = self.size;
        self.size = 0;
        Iter::new(self.inner.drain(..), size, std::convert::identity)
    }

    /// Creates an iterator which uses a closure to determine if an element should be removed.
    ///
    /// If the closure returns true, the element is removed from the map and yielded.
    /// If the closure returns false, or panics, the element remains in the map and will not be
    /// yielded.
    ///
    /// Note that `extract_if` lets you mutate every value in the filter closure, regardless of
    /// whether you choose to keep or remove it.
    ///
    /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
    /// or the iteration short-circuits, then the remaining elements will be retained.
    /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
    ///
    /// [`retain`]: EnumMap::retain
    ///
    /// # Examples
    ///
    /// Splitting a map into even and odd keys, reusing the original map:
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap::<Ordering, i32> = EnumMap::from([
    ///     (Ordering::Less, -5),
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Greater, 10),
    /// ]);
    /// let extracted: EnumMap::<Ordering, i32> = map.extract_if(|_k, v| *v >= 0).collect();
    ///
    /// let mut positives = extracted.values().copied().collect::<Vec<_>>();
    /// let mut negatives = map.values().copied().collect::<Vec<_>>();
    /// positives.sort();
    /// negatives.sort();
    ///
    /// assert_eq!(positives, vec![1, 10]);
    /// assert_eq!(negatives, vec![-5]);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn extract_if<'a, F>(&'a mut self, pred: F) -> ExtractIf<'a, K, V, F>
    where
        F: 'a + FnMut(K, &mut V) -> bool,
    {
        ExtractIf::new(self.inner.iter_mut(), &mut self.size, pred)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(k, &mut v)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap::<Ordering, i32> = EnumMap::from([
    ///     (Ordering::Less, -5),
    ///     (Ordering::Equal, 1),
    ///     (Ordering::Greater, 10),
    /// ]);
    /// map.retain(|_key, val| *val >= 0);
    /// assert_eq!(map.len(), 2);
    /// ```
    ///
    /// # Performance
    ///
    /// In the current implementation, this operation takes O(capacity) time
    /// instead of O(len) because it internally visits empty buckets too.
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

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut a = EnumMap::new();
    /// a.insert(Ordering::Less, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn clear(&mut self) {
        self.size = 0;
        self.inner.fill_with(Default::default);
    }

    #[inline]
    fn allocate(&mut self) {
        if self.inner.is_empty() {
            self.inner.resize_with(K::SIZE, Default::default);
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut ords = EnumMap::new();
    ///
    /// for ord in [Ordering::Less, Ordering::Equal, Ordering::Less, Ordering::Less] {
    ///     ords.entry(ord).and_modify(|counter| *counter += 1).or_insert(1);
    /// }
    ///
    /// assert_eq!(ords[Ordering::Less], 3);
    /// assert_eq!(ords[Ordering::Equal], 1);
    /// assert_eq!(ords.get(Ordering::Greater), None);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        self.allocate();
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

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::new();
    /// map.insert(Ordering::Less, "a");
    /// assert_eq!(map.get(Ordering::Less), Some(&"a"));
    /// assert_eq!(map.get(Ordering::Equal), None);
    /// ```
    #[inline]
    pub fn get(&self, k: K) -> Option<&V> {
        self.inner.get(k.index()).and_then(Option::as_ref)
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::new();
    /// map.insert(Ordering::Less, "a");
    /// assert_eq!(map.contains_key(Ordering::Less), true);
    /// assert_eq!(map.contains_key(Ordering::Equal), false);
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn contains_key(&self, k: K) -> bool {
        matches!(self.inner.get(k.index()), Some(Some(_)))
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but
    /// [`Hash`] and [`Eq`] on the borrowed form *must* match those for
    /// the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::new();
    /// map.insert(Ordering::Less, "a");
    /// if let Some(x) = map.get_mut(Ordering::Less) {
    ///     *x = "b";
    /// }
    /// assert_eq!(map[Ordering::Less], "b");
    /// ```
    #[inline]
    pub fn get_mut(&mut self, k: K) -> Option<&mut V> {
        self.inner.get_mut(k.index()).and_then(Option::as_mut)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::new();
    /// assert_eq!(map.insert(Ordering::Less, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(Ordering::Less, "b");
    /// assert_eq!(map.insert(Ordering::Less, "c"), Some("b"));
    /// assert_eq!(map[Ordering::Less], "c");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.allocate();
        let old_val = self.inner[k.index()].replace(v);
        if old_val.is_none() {
            self.size += 1;
        }
        old_val
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map = EnumMap::new();
    /// map.insert(Ordering::Less, "a");
    /// assert_eq!(map.remove(Ordering::Less), Some("a"));
    /// assert_eq!(map.remove(Ordering::Less), None);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(&mut self, k: K) -> Option<V> {
        let old_val = self.inner.get_mut(k.index())?.take();
        if old_val.is_some() {
            self.size -= 1;
        }
        old_val
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

impl<K: Enum, V> FromIterator<(K, V)> for EnumMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut inner: Vec<Option<V>> = Vec::with_capacity(K::SIZE);
        inner.resize_with(K::SIZE, Default::default);
        let mut size = 0;
        for (key, val) in iter {
            size += 1;
            inner[key.index()] = Some(val);
        }
        Self {
            inner,
            size,
            marker: PhantomData,
        }
    }
}

impl<K: Enum, V, const N: usize> From<[(K, V); N]> for EnumMap<K, V> {
    fn from(value: [(K, V); N]) -> Self {
        Self::from_iter(value)
    }
}
