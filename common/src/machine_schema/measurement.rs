use serde::Deserialize ;

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    Boolean,
    Integer{
        statistics: Statistics,
    },
    Float {
        semantic: FloatSemantic,
        statistics: Statistics,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Statistics {
    /// Track the minimum observed value each sampling cycle. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub min: bool,
    /// Track the minimum observed value each sampling cycle. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub max: bool,
}

// --- deserialize implemenations ---
use serde::de::{Error, Deserializer};
use crate::machine_schema::{FloatSemantic, value_type};

use super::ValueType;

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
        let value_t = value_type::parse(tag)
            .map_err(Error::custom)?;
        
        let value = tagged.value;

        match value_t {
            ValueType::Enum => {
                Err(Error::custom("enums are not supported for measurements"))
            },
            ValueType::String => {
                Err(Error::custom("strings are not supported for measurements"))
            }
            ValueType::Boolean => {
                let BooleanHelper { nullable } = BooleanHelper::deserialize(value)
                    .map_err(Error::custom)?;

                 Ok(Value { kind: ValueKind::Boolean, nullable })
            }
            ValueType::Integer => {
                let NumericHelper { nullable, statistics } = NumericHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::Integer { statistics }, nullable })
            }
            ValueType::Float(semantic) => {
                let NumericHelper { nullable, statistics } = NumericHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(Value { kind: ValueKind::Float { semantic, statistics }, nullable })
            },
            other => Err(Error::custom(format!("Unsupported type: {other:?}"))),
        }
    }
}

// --- boolean ---
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BooleanHelper {
    #[serde(default)]
    pub nullable: bool,
}

// --- numeric ---
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericHelper {
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub statistics: Statistics,
}
