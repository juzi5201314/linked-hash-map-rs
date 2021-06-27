use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::LinkedHashMap;

impl<K, V> Serialize for LinkedHashMap<K, V>
where
    K: Serialize + Hash + Eq,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut ser_map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self.iter() {
            ser_map.serialize_entry(k, v)?
        }
        ser_map.end()
    }
}

pub struct LinkedHashMapVisitor<K, V> {
    marker: PhantomData<LinkedHashMap<K, V>>,
}

impl<K, V> LinkedHashMapVisitor<K, V> {
    pub fn new() -> Self {
        LinkedHashMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for LinkedHashMapVisitor<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    type Value = LinkedHashMap<K, V>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        write!(formatter, "a linked hash map")
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(LinkedHashMap::new())
    }

    #[inline]
    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut values = LinkedHashMap::with_capacity(map.size_hint().unwrap_or(0));

        while let Some((key, value)) = map.next_entry()? {
            values.insert(key, value);
        }

        Ok(values)
    }
}

impl<'de, K, V> Deserialize<'de> for LinkedHashMap<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<LinkedHashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(LinkedHashMapVisitor::new())
    }
}
