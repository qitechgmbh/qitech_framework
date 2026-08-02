use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::MeasurementDefinition;
use crate::schema::MeasurementKind;
use crate::schema::MeasurementStatistics;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct MeasurementInfoRaw(pub MeasurementDefinition);

impl<'de> Deserialize<'de> for MeasurementInfoRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MeasurementValueVisitor;

        impl<'de> Visitor<'de> for MeasurementValueVisitor {
            type Value = MeasurementInfoRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged measurement value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Keyword::from_str(&tag).map_err(A::Error::custom)? {
                    Keyword::Boolean => {
                        let BooleanHelper { nullable } = variant.newtype_variant()?;

                        Ok(MeasurementInfoRaw(MeasurementDefinition {
                            kind: MeasurementKind::Boolean,
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Integer => {
                        let NumericHelper {
                            nullable,
                            statistics,
                        } = variant.newtype_variant()?;

                        Ok(MeasurementInfoRaw(MeasurementDefinition {
                            kind: MeasurementKind::Integer { statistics },
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Float(semantic) => {
                        let NumericHelper {
                            nullable,
                            statistics,
                        } = variant.newtype_variant()?;

                        Ok(MeasurementInfoRaw(MeasurementDefinition {
                            kind: MeasurementKind::Float {
                                semantic,
                                statistics,
                            },
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    Keyword::Enum => {
                        Err(A::Error::custom("enums are not supported for measurements"))
                    }

                    Keyword::String => Err(A::Error::custom(
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
struct NumericHelper {
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub statistics: MeasurementStatistics,
}
