use serde::Deserialize ;

#[derive(Debug, Clone)]
pub struct MeasurementValue {
    pub kind: MeasurementValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum MeasurementValueKind {
    Boolean,
    Integer{
        statistics: MeasurementStatistics,
    },
    Float {
        semantic: FloatSemantic,
        statistics: MeasurementStatistics,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementStatistics {
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
use crate::machine_schema::{FloatSemantic, r#type};

use super::Type;

impl<'de> Deserialize<'de> for MeasurementValue {
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
                Err(Error::custom("enums are not supported for measurements"))
            },
            Type::String => {
                Err(Error::custom("strings are not supported for measurements"))
            }
            Type::Boolean => {
                let BooleanHelper { nullable } = BooleanHelper::deserialize(value)
                    .map_err(Error::custom)?;

                 Ok(MeasurementValue { kind: MeasurementValueKind::Boolean, nullable })
            }
            Type::Integer => {
                let NumericHelper { nullable, statistics } = NumericHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(MeasurementValue { kind: MeasurementValueKind::Integer { statistics }, nullable })
            }
            Type::Float(semantic) => {
                let NumericHelper { nullable, statistics } = NumericHelper::deserialize(value)
                    .map_err(Error::custom)?;

                Ok(MeasurementValue { kind: MeasurementValueKind::Float { semantic, statistics }, nullable })
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
    pub statistics: MeasurementStatistics,
}
