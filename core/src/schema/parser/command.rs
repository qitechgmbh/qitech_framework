use core::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::CommandDefinition;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct CommandDefinitionRaw(pub CommandDefinition);

impl<'de> Deserialize<'de> for CommandDefinitionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = CommandDefinitionRaw;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let value_type = Keyword::from_str(&tag).map_err(A::Error::custom)?;

                if !matches!(value_type, Keyword::Command) {
                    return Err(Error::custom(format!(
                        "expected !command, received: !{tag}."
                    )));
                }

                variant.unit_variant()?;

                Ok(CommandDefinitionRaw(CommandDefinition {
                    metadata: Default::default(),
                }))
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("!command")
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}
