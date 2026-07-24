use serde::Deserialize;

use super::FloatSemantic;

#[derive(Debug, Clone)]
pub struct MeasurementValue {
    pub kind: MeasurementValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum MeasurementValueKind {
    Boolean,
    Integer {
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
use std::fmt;
use std::str::FromStr;

use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use super::Type;

impl<'de> Deserialize<'de> for MeasurementValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MeasurementValueVisitor;

        impl<'de> Visitor<'de> for MeasurementValueVisitor {
            type Value = MeasurementValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged measurement value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Type::from_str(&tag).map_err(A::Error::custom)? {
                    Type::Boolean => {
                        let BooleanHelper { nullable } = variant.newtype_variant()?;

                        Ok(MeasurementValue {
                            kind: MeasurementValueKind::Boolean,
                            nullable,
                        })
                    }

                    Type::Integer => {
                        let NumericHelper {
                            nullable,
                            statistics,
                        } = variant.newtype_variant()?;

                        Ok(MeasurementValue {
                            kind: MeasurementValueKind::Integer { statistics },
                            nullable,
                        })
                    }

                    Type::Float(semantic) => {
                        let NumericHelper {
                            nullable,
                            statistics,
                        } = variant.newtype_variant()?;

                        Ok(MeasurementValue {
                            kind: MeasurementValueKind::Float {
                                semantic,
                                statistics,
                            },
                            nullable,
                        })
                    }

                    Type::Enum => Err(A::Error::custom("enums are not supported for measurements")),

                    Type::String => Err(A::Error::custom(
                        "strings are not supported for measurements",
                    )),

                    other => Err(A::Error::custom(format!(
                        "unsupported measurement type: {other:?}"
                    ))),
                }
            }
        }

        deserializer.deserialize_any(MeasurementValueVisitor)
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
