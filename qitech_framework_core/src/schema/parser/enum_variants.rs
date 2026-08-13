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
                let mut variants = EnumVariants::new();
                let mut index = 0i64;

                while let Some(name) = seq.next_element::<String>()? {
                    insert(&mut variants, name, index).map_err(Error::custom)?;
                    index += 1;
                }

                finish(variants)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut variants = EnumVariants::new();

                while let Some((name, value)) = map.next_entry::<String, i64>()? {
                    insert(&mut variants, name, value).map_err(Error::custom)?;
                }

                finish(variants)
            }
        }

        deserializer.deserialize_any(EnumVariantsVisitor)
    }
}

// --- utils ---
fn insert(variants: &mut EnumVariants, name: String, value: i64) -> Result<(), String> {
    if variants.values.contains_key(&name) {
        return Err(format!("duplicate enum key {:?}", name));
    }

    if variants.reverse.contains_key(&value) {
        return Err(format!("duplicate enum value {:?}", value));
    }

    variants.values.insert(name.clone(), value);
    variants.reverse.insert(value, name);
    Ok(())
}

fn finish<E>(variants: EnumVariants) -> Result<EnumVariantsRaw, E>
where
    E: Error,
{
    if variants.is_empty() {
        Err(E::custom("enum cannot be empty"))
    } else {
        Ok(EnumVariantsRaw(variants))
    }
}
