use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::StatePropertyDefinition;
use crate::schema::StatePropertyKind;
use crate::schema::parser::enum_variants::EnumVariantsRaw;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct StatePropertyDefinitionRaw(pub StatePropertyDefinition);

impl<'de> Deserialize<'de> for StatePropertyDefinitionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatePropertyVisitor;

        impl<'de> Visitor<'de> for StatePropertyVisitor {
            type Value = StatePropertyDefinitionRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged state property value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Keyword::from_str(&tag).map_err(A::Error::custom)? {
                    Keyword::Enum => {
                        let EnumValueHelper { nullable, variants } = variant.newtype_variant()?;

                        let variants = variants.0;
                        Ok(StatePropertyDefinitionRaw(StatePropertyDefinition {
                            kind: StatePropertyKind::Enum { variants },
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::String => {
                        let SimpleValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyDefinitionRaw(StatePropertyDefinition {
                            kind: StatePropertyKind::String,
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Boolean => {
                        let SimpleValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyDefinitionRaw(StatePropertyDefinition {
                            kind: StatePropertyKind::Boolean,
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Integer => {
                        let SimpleValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyDefinitionRaw(StatePropertyDefinition {
                            kind: StatePropertyKind::Integer,
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Float(semantic) => {
                        let SimpleValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyDefinitionRaw(StatePropertyDefinition {
                            kind: StatePropertyKind::Float { semantic },
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    other => Err(A::Error::custom(format!(
                        "unsupported state property type: {other:?}"
                    ))),
                }
            }
        }

        deserializer.deserialize_any(StatePropertyVisitor)
    }
}

// --- enum ---
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumValueHelper {
    #[serde(default)]
    nullable: bool,
    variants: EnumVariantsRaw,
}

// --- simple ---
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SimpleValueHelper {
    #[serde(default)]
    nullable: bool,
}
