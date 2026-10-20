use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::ser::Serializer;

use crate::schema::Map;
use crate::schema::StringMap;

/// Named enum variants with stable integer values.
///
/// Invariants: never empty, names and values are both unique.
///
/// Wire format is a plain map of name to integer, e.g. `{"Auto": 0, "Manual": 1}`.
/// The reverse lookup is rebuilt and the invariants are re-checked on deserialization.
#[derive(Debug, Clone)]
pub struct EnumVariants {
    values: StringMap<i64>,
    reverse: Map<i64, String>,
}

impl EnumVariants {
    pub fn first(&self) -> (&String, &i64) {
        self.values.first().expect("Cannot be empty")
    }

    pub fn last(&self) -> (&String, &i64) {
        self.values.last().expect("Cannot be empty")
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.values.get(name).copied()
    }

    pub fn get_name(&self, value: i64) -> Option<&str> {
        self.reverse.get(&value).map(String::as_str)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, i64> {
        self.values.iter()
    }

    pub fn list(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Builds variants from `(name, value)` pairs, enforcing all invariants.
    pub(crate) fn try_from_iter<I>(iter: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (String, i64)>,
    {
        let mut variants = Self {
            values: Default::default(),
            reverse: Default::default(),
        };

        for (name, value) in iter {
            variants.insert(name, value)?;
        }

        if variants.is_empty() {
            return Err("enum cannot be empty".to_owned());
        }

        Ok(variants)
    }

    fn insert(&mut self, name: String, value: i64) -> Result<(), String> {
        if self.values.contains_key(&name) {
            return Err(format!("duplicate enum key {name:?}"));
        }

        if self.reverse.contains_key(&value) {
            return Err(format!("duplicate enum value {value:?}"));
        }

        self.values.insert(name.clone(), value);
        self.reverse.insert(value, name);
        Ok(())
    }
}

impl<'a> IntoIterator for &'a EnumVariants {
    type Item = (&'a String, &'a i64);
    type IntoIter = indexmap::map::Iter<'a, String, i64>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl IntoIterator for EnumVariants {
    type Item = (String, i64);
    type IntoIter = indexmap::map::IntoIter<String, i64>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

// --- serde ---
impl Serialize for EnumVariants {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.values.len()))?;

        for (name, value) in &self.values {
            map.serialize_entry(name, value)?;
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for EnumVariants {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnumVariantsVisitor;

        impl<'de> Visitor<'de> for EnumVariantsVisitor {
            type Value = EnumVariants;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a non-empty map of enum name to integer")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));

                while let Some(entry) = map.next_entry::<String, i64>()? {
                    entries.push(entry);
                }

                EnumVariants::try_from_iter(entries).map_err(Error::custom)
            }
        }

        // deserialize_map instead of deserialize_any: postcard is not self-describing
        deserializer.deserialize_map(EnumVariantsVisitor)
    }
}
