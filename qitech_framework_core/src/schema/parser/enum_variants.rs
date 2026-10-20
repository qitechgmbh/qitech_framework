use core::fmt;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use crate::schema::EnumVariants;

#[derive(Debug)]
pub struct EnumVariantsRaw(pub EnumVariants);

impl<'de> Deserialize<'de> for EnumVariantsRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnumVariantsVisitor;

        impl<'de> Visitor<'de> for EnumVariantsVisitor {
            type Value = EnumVariantsRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of enum names or a map of enum name to integer")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = Vec::new();
                let mut index = 0i64;

                while let Some(name) = seq.next_element::<String>()? {
                    entries.push((name, index));
                    index += 1;
                }

                finish(entries)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();

                while let Some(entry) = map.next_entry::<String, i64>()? {
                    entries.push(entry);
                }

                finish(entries)
            }
        }

        deserializer.deserialize_any(EnumVariantsVisitor)
    }
}

// --- utils ---
fn finish<E>(entries: Vec<(String, i64)>) -> Result<EnumVariantsRaw, E>
where
    E: Error,
{
    EnumVariants::try_from_iter(entries)
        .map(EnumVariantsRaw)
        .map_err(E::custom)
}
