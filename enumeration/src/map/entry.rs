use crate::enum_trait::Enum;

pub enum Entry<'a, K, V> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

pub struct OccupiedEntry<'a, K, V> {
    pub(super) key: K,
    pub(super) value: &'a mut Option<V>,
    pub(super) size: &'a mut usize,
}

impl<'a, K: Enum, V> OccupiedEntry<'a, K, V> {
    pub fn key(&self) -> K {
        self.key
    }

    pub fn remove_entry(self) -> (K, V) {
        (self.key, self.remove())
    }

    pub fn get(&self) -> &V {
        self.value.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut V {
        self.value.as_mut().unwrap()
    }

    pub fn into_mut(self) -> &'a mut V {
        self.value.as_mut().unwrap()
    }

    pub fn insert(&mut self, value: V) -> V {
        self.value.replace(value).unwrap()
    }

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
    pub fn key(&self) -> K {
        self.key
    }

    pub fn insert(self, value: V) -> &'a mut V {
        *self.size += 1;
        self.value.replace(value);
        self.value.as_mut().unwrap()
    }
}
