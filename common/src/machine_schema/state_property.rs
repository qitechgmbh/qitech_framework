use crate::machine_schema::r#type;

use super::{EnumVariants, FloatSemantic};

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
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
use serde::{Deserialize, de::{Error, Deserializer}};
use super::Type;

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = yaml_serde::Value::deserialize(deserializer)?;

        let yaml_serde::Value::Tagged(tagged) = value else {
            return Err(Error::custom("expected tagged value"));
        };

        // skip te '!'
        let tag = &tagged.tag.to_string()[1..];

        // read the value type / tag
        let value_t = r#type::parse(tag)
            .map_err(Error::custom)?;
        
        let value = tagged.value;

        match value_t {
            Type::Enum => {
                let EnumValueHelper { nullable, variants } = EnumValueHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::Enum { variants }, nullable })
            },
            Type::String => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::String, nullable })
            }
            Type::Boolean => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::Boolean, nullable })
            }
            Type::Integer => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::String, nullable })
            }
            Type::Float(semantic) => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::Float { semantic }, nullable })
            },
            other => Err(Error::custom(format!("Unsupported type: {other:?}"))),
        }
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
