use std::fmt::{self, Formatter};
use std::iter::FromIterator;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Enum, EnumMap};

impl<K, V> Serialize for EnumMap<K, V>
where
    K: Enum + Serialize + Copy + Ord,
    V: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self)
    }
}

impl<K: Enum + Copy + Ord, V> FromIterator<(K, V)> for EnumMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut this = EnumMap::new();
        for (k, v) in iter {
            this.insert(k, v);
        }
        this
    }
}

impl<'de, K, V> Deserialize<'de> for EnumMap<K, V>
where
    K: Enum + Deserialize<'de> + Copy + Ord,
    V: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MapVisitor<K, V> {
            marker: PhantomData<EnumMap<K, V>>,
        }

        impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
        where
            K: Enum + Deserialize<'de>,
            V: Deserialize<'de>,
        {
            type Value = EnumMap<K, V>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = EnumMap::new();
                while let Some((k, v)) = map.next_entry()? {
                    values.insert(k, v);
                }
                Ok(values)
            }
        }

        let visitor = MapVisitor {
            marker: PhantomData,
        };
        deserializer.deserialize_map(visitor)
    }
}
