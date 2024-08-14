use std::fmt::{self, Formatter};
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Enum, EnumMap, EnumSet};

impl<T> Serialize for EnumSet<T>
where
    T: Enum + Serialize,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(*self)
    }
}

impl<'de, T> Deserialize<'de> for EnumSet<T>
where
    T: Enum + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SeqVisitor<T: Enum> {
            marker: PhantomData<T>,
        }

        impl<'de, T> serde::de::Visitor<'de> for SeqVisitor<T>
        where
            T: Enum + Deserialize<'de>,
        {
            type Value = EnumSet<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            #[inline]
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values: EnumSet<T> = EnumSet::new();

                while let Some(value) = seq.next_element()? {
                    values.insert(value);
                }

                Ok(values)
            }
        }

        let visitor = SeqVisitor {
            marker: PhantomData,
        };
        deserializer.deserialize_seq(visitor)
    }
}

impl<K, V> Serialize for EnumMap<K, V>
where
    K: Enum + Serialize,
    V: Serialize,
{
    #[cfg_attr(feature = "inline-more", inline)]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self)
    }
}

impl<'de, K, V> Deserialize<'de> for EnumMap<K, V>
where
    K: Enum + Deserialize<'de>,
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

#[cfg(test)]
mod tests {
    use crate::enums;
    use serde::de::DeserializeOwned;

    use super::*;

    #[rustfmt::skip] #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
    enum DemoEnum { A, B, C, D, E, F, G, H, I, J }

    impl Serialize for DemoEnum {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            usize::serialize(&self.index(), serializer)
        }
    }

    impl<'de> Deserialize<'de> for DemoEnum {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let value = usize::deserialize(deserializer)?;
            Ok(Self::from_index(value).unwrap())
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_roundtrip_eq<T: Eq + std::fmt::Debug + Serialize + DeserializeOwned>(value: T) {
        let serialized = serde_json::to_value(&value).unwrap();
        let deserialized: T = serde_json::from_value(serialized).unwrap();
        assert_eq!(value, deserialized);
    }
    // EnumSet tests

    #[test]
    fn set_round_trip() {
        assert_roundtrip_eq(enums![DemoEnum::A, DemoEnum::E, DemoEnum::I]);
    }

    #[test]
    fn map_round_trip() {
        let mut map: EnumMap<DemoEnum, String> = EnumMap::new();
        map.insert(DemoEnum::B, "foo".to_owned());
        map.insert(DemoEnum::D, "bar".to_owned());
        assert_roundtrip_eq(map);
    }
}
