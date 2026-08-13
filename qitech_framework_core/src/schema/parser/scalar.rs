use core::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::FloatSemantic;
use crate::schema::ScalarPropertyDefinition;
use crate::schema::ScalarPropertyKind;
use crate::schema::parser::enum_variants::EnumVariantsRaw;

#[derive(Debug)]
pub struct ScalarPropertyDefinitionRaw(pub ScalarPropertyDefinition);

impl<'de> Deserialize<'de> for ScalarPropertyDefinitionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ScalarPropertyDefinition;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged scalar value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let nullable = tag.starts_with("?");
                let tag = if nullable { &tag[1..] } else { &tag };

                if let Ok(semantic) = FloatSemantic::from_str(tag) {
                    variant.unit_variant()?;

                    return Ok(ScalarPropertyDefinition {
                        kind: ScalarPropertyKind::Float { semantic },
                        nullable,
                        metadata: Default::default(),
                    });
                }

                match tag {
                    "enum" => {
                        let EnumVariantsRaw(variants) = variant.newtype_variant()?;

                        Ok(ScalarPropertyDefinition {
                            kind: ScalarPropertyKind::Enum { variants },
                            nullable,
                            metadata: Default::default(),
                        })
                    }

                    "string" => {
                        variant.unit_variant()?;

                        Ok(ScalarPropertyDefinition {
                            kind: ScalarPropertyKind::String,
                            nullable,
                            metadata: Default::default(),
                        })
                    }

                    "integer" => {
                        variant.unit_variant()?;

                        Ok(ScalarPropertyDefinition {
                            kind: ScalarPropertyKind::Integer,
                            nullable,
                            metadata: Default::default(),
                        })
                    }

                    "boolean" => {
                        variant.unit_variant()?;

                        Ok(ScalarPropertyDefinition {
                            kind: ScalarPropertyKind::Boolean,
                            nullable,
                            metadata: Default::default(),
                        })
                    }

                    other => Err(A::Error::custom(format!(
                        "unsupported scalar type: {other:?}"
                    ))),
                }
            }
        }

        let definition = deserializer.deserialize_any(ValueVisitor)?;
        Ok(ScalarPropertyDefinitionRaw(definition))
    }
}
