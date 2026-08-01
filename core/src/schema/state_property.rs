use super::EnumVariants;
use super::FloatSemantic;

#[derive(Debug, Clone, Serialize)]
pub struct StatePropertyValue {
    pub kind: StatePropertyValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum StatePropertyValueKind {
    Enum {
        /// The set of allowed variants for this value. Required.
        variants: EnumVariants,
    },
    String,
    Boolean,
    Integer,
    Float {
        /// Representation of the float. E.g. plain, fraction, millimeter
        semantic: FloatSemantic,
    },
}

// --- deserialize implemenations ---
use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use super::Type;

impl<'de> Deserialize<'de> for StatePropertyValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatePropertyVisitor;

        impl<'de> Visitor<'de> for StatePropertyVisitor {
            type Value = StatePropertyValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged state property value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Type::from_str(&tag).map_err(A::Error::custom)? {
                    Type::Enum => {
                        let EnumValueHelper { nullable, variants } = variant.newtype_variant()?;

                        Ok(StatePropertyValue {
                            kind: StatePropertyValueKind::Enum { variants },
                            nullable,
                        })
                    }

                    Type::String => {
                        let OtherValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyValue {
                            kind: StatePropertyValueKind::String,
                            nullable,
                        })
                    }

                    Type::Boolean => {
                        let OtherValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyValue {
                            kind: StatePropertyValueKind::Boolean,
                            nullable,
                        })
                    }

                    Type::Integer => {
                        let OtherValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyValue {
                            kind: StatePropertyValueKind::Integer,
                            nullable,
                        })
                    }

                    Type::Float(semantic) => {
                        let OtherValueHelper { nullable } = variant.newtype_variant()?;

                        Ok(StatePropertyValue {
                            kind: StatePropertyValueKind::Float { semantic },
                            nullable,
                        })
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
pub struct EnumValueHelper {
    #[serde(default)]
    nullable: bool,
    variants: EnumVariants,
}

// --- scalar ---
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtherValueHelper {
    #[serde(default)]
    nullable: bool,
}
