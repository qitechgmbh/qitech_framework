use core::fmt;

use serde::Deserialize;
use serde::de;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::CommandDefinition;

#[derive(Debug)]
pub struct CommandDefinitionRaw(pub CommandDefinition);

impl<'de> Deserialize<'de> for CommandDefinitionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CommandVisitor;

        impl<'de> Visitor<'de> for CommandVisitor {
            type Value = CommandDefinition;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                if tag != "command" {
                    return Err(de::Error::custom(format!(
                        "expected !command, received: !{tag}."
                    )));
                }

                variant.unit_variant()?;
                Ok(CommandDefinition {
                    metadata: Default::default(),
                })
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("!command")
            }
        }

        let definition = deserializer.deserialize_any(CommandVisitor)?;
        Ok(CommandDefinitionRaw(definition))
    }
}
