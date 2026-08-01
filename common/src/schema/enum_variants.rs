use super::Map;
use super::StringMap;

#[derive(Debug, Clone, Serialize)]
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

    fn new() -> Self {
        Self {
            values: Default::default(),
            reverse: Default::default(),
        }
    }

    /// can only be invoked here, only intended for building
    fn insert(&mut self, name: String, value: i64) -> Result<(), String> {
        if self.values.contains_key(&name) {
            return Err(format!("duplicate enum key {:?}", name));
        }

        if self.reverse.contains_key(&value) {
            return Err(format!("duplicate enum value {:?}", value));
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

// --- deserialize impl --
use std::fmt::Formatter;
use std::fmt::{self};

use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

impl<'de> Deserialize<'de> for EnumVariants {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnumVariantsVisitor;

        impl<'de> Visitor<'de> for EnumVariantsVisitor {
            type Value = EnumVariants;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of enum names or a map of enum name to integer")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut variants = EnumVariants::new();
                let mut index = 0i64;

                while let Some(name) = seq.next_element::<String>()? {
                    variants.insert(name, index).map_err(Error::custom)?;
                    index += 1;
                }

                finish_variants(variants)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut variants = EnumVariants::new();

                while let Some((name, value)) = map.next_entry::<String, i64>()? {
                    variants.insert(name, value).map_err(Error::custom)?;
                }

                finish_variants(variants)
            }
        }

        deserializer.deserialize_any(EnumVariantsVisitor)
    }
}

fn finish_variants<E>(variants: EnumVariants) -> Result<EnumVariants, E>
where
    E: Error,
{
    if variants.is_empty() {
        Err(E::custom("enum cannot be empty"))
    } else {
        Ok(variants)
    }
}
