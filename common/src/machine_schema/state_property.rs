use super::{EnumVariants, FloatSemantic};

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    Enum(EnumValue),
    String,
    Boolean,
    Integer,
    Float(FloatSemantic),
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    /// The set of allowed variants for this value. Required.
    pub variants: EnumVariants,
}

// --- deserialize implemenations ---
use serde::{Deserialize, de::{self, Deserializer}};
use super::ValueType;

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = yaml_serde::Value::deserialize(deserializer)?;

        let yaml_serde::Value::Tagged(tagged) = value else {
            return Err(de::Error::custom("expected tagged value"));
        };

        // read the value type / tag
        let value_t = yaml_serde::from_str::<ValueType>(&tagged.tag.to_string())
            .map_err(de::Error::custom)?;

        let value = tagged.value;

        match value_t {
            ValueType::Enum => {
                let EnumValueHelper { nullable, variants } = EnumValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                Ok(Value { kind: ValueKind::Enum(EnumValue { variants }), nullable })
            },
            ValueType::String => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                Ok(Value { kind: ValueKind::String, nullable })
            }
            ValueType::Boolean => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                Ok(Value { kind: ValueKind::Boolean, nullable })
            }
            ValueType::Integer => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                Ok(Value { kind: ValueKind::String, nullable })
            }
            ValueType::Float(semantic) => {
                let OtherValueHelper { nullable } = OtherValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                Ok(Value { kind: ValueKind::Float(semantic), nullable })
            },
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
