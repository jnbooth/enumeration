use crate::enumerate::Enum;

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`EnumMap`].
///
/// [`EnumMap`]: crate::EnumMap
/// [`entry`]: crate::EnumMap::entry
pub enum Entry<'a, K, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K: Enum, V> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    ///
    /// map.entry(Ordering::Less).or_insert(3);
    /// assert_eq!(map[Ordering::Less], 3);
    ///
    /// *map.entry(Ordering::Less).or_insert(10) *= 2;
    /// assert_eq!(map[Ordering::Less], 6);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default),
        }
    }
    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, &str> = EnumMap::new();
    /// let value = "hoho";
    ///
    /// map.entry(Ordering::Less).or_insert_with(|| value);
    ///
    /// assert_eq!(map[Ordering::Less], "hoho");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of the default function.
    /// This method allows for generating key-derived values for insertion by providing the default
    /// function with the key that was used during the `.entry(key)` method call.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, i8> = EnumMap::new();
    ///
    /// map.entry(Ordering::Less).or_insert_with_key(|key| key as i8);
    ///
    /// assert_eq!(map[Ordering::Less], -1);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert_with_key<F: FnOnce(K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this entry's key.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// assert_eq!(map.entry(Ordering::Less).key(), Ordering::Less);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn key(&self) -> K {
        match *self {
            Self::Occupied(ref entry) => entry.key(),
            Self::Vacant(ref entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    ///
    /// map.entry(Ordering::Less)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map[Ordering::Less], 42);
    ///
    /// map.entry(Ordering::Less)
    ///    .and_modify(|e| { *e += 1 })
    ///    .or_insert(42);
    /// assert_eq!(map[Ordering::Less], 43);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

/// A view into an occupied entry in a `EnumMap`.
/// It is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K, V> {
    pub(super) key: K,
    pub(super) value: &'a mut Option<V>,
    pub(super) size: &'a mut usize,
}

impl<'a, K: Enum, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the key in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    /// assert_eq!(map.entry(Ordering::Less).key(), Ordering::Less);
    /// ```
    #[inline]
    pub fn key(&self) -> K {
        self.key
    }

    /// Take the ownership of the key and value from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry(Ordering::Less) {
    ///     // We delete the entry from the map.
    ///     o.remove_entry();
    /// }
    ///
    /// assert_eq!(map.contains_key(Ordering::Less), false);
    /// ```
    #[inline]
    pub fn remove_entry(self) -> (K, V) {
        (self.key, self.remove())
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry(Ordering::Less) {
    ///     assert_eq!(o.get(), &12);
    /// }
    /// ```
    #[inline]
    pub fn get(&self) -> &V {
        self.value.as_ref().unwrap()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// assert_eq!(map[Ordering::Less], 12);
    /// if let Entry::Occupied(mut o) = map.entry(Ordering::Less) {
    ///     *o.get_mut() += 10;
    ///     assert_eq!(*o.get(), 22);
    ///
    ///     // We can use the same Entry multiple times.
    ///     *o.get_mut() += 2;
    /// }
    ///
    /// assert_eq!(map[Ordering::Less], 24);
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        self.value.as_mut().unwrap()
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// assert_eq!(map[Ordering::Less], 12);
    /// if let Entry::Occupied(o) = map.entry(Ordering::Less) {
    ///     *o.into_mut() += 10;
    /// }
    ///
    /// assert_eq!(map[Ordering::Less], 22);
    /// ```
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        self.value.as_mut().unwrap()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// if let Entry::Occupied(mut o) = map.entry(Ordering::Less) {
    ///     assert_eq!(o.insert(15), 12);
    /// }
    ///
    /// assert_eq!(map[Ordering::Less], 15);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(&mut self, value: V) -> V {
        self.value.replace(value).unwrap()
    }

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// map.entry(Ordering::Less).or_insert(12);
    ///
    /// if let Entry::Occupied(o) = map.entry(Ordering::Less) {
    ///     assert_eq!(o.remove(), 12);
    /// }
    ///
    /// assert_eq!(map.contains_key(Ordering::Less), false);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(self) -> V {
        *self.size -= 1;
        self.value.take().unwrap()
    }
}

pub struct VacantEntry<'a, K, V> {
    pub(super) key: K,
    pub(super) value: &'a mut Option<V>,
    pub(super) size: &'a mut usize,
}

impl<'a, K: Enum, V> VacantEntry<'a, K, V> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    /// assert_eq!(map.entry(Ordering::Less).key(), Ordering::Less);
    /// ```
    #[inline]
    pub fn key(&self) -> K {
        self.key
    }

    /// Sets the value of the entry with the `VacantEntry`'s key,
    /// and returns a mutable reference to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cmp::Ordering;
    /// use enumeration::EnumMap;
    /// use enumeration::map::Entry;
    ///
    /// let mut map: EnumMap<Ordering, u32> = EnumMap::new();
    ///
    /// if let Entry::Vacant(o) = map.entry(Ordering::Less) {
    ///     o.insert(37);
    /// }
    /// assert_eq!(map[Ordering::Less], 37);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self, value: V) -> &'a mut V {
        *self.size += 1;
        self.value.replace(value);
        self.value.as_mut().unwrap()
    }
}
