use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::FloatSemantic;
use crate::schema::MeasurementDefinition;
use crate::schema::MeasurementKind;
use crate::schema::MeasurementStatistics;

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
                #[derive(Debug, Clone, Deserialize)]
                #[serde(deny_unknown_fields)]
                struct NumericHelper {
                    #[serde(default)]
                    pub statistics: MeasurementStatistics,
                }

                let (tag, variant) = data.variant::<String>()?;

                let nullable = tag.starts_with("?");
                let tag = if nullable { &tag[1..] } else { &tag };

                if let Ok(semantic) = FloatSemantic::from_str(tag) {
                    let NumericHelper { statistics } = variant.newtype_variant()?;

                    return Ok(MeasurementInfoRaw(MeasurementDefinition {
                        kind: MeasurementKind::Float {
                            semantic,
                            statistics,
                        },
                        nullable,
                        metadata: Default::default(),
                    }));
                }

                match tag {
                    "integer" => {
                        let NumericHelper { statistics } = variant.newtype_variant()?;

                        Ok(MeasurementInfoRaw(MeasurementDefinition {
                            kind: MeasurementKind::Integer { statistics },
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    "boolean" => {
                        variant.unit_variant()?;
                        Ok(MeasurementInfoRaw(MeasurementDefinition {
                            kind: MeasurementKind::Boolean,
                            nullable,
                            metadata: Default::default(),
                        }))
                    }

                    other => Err(A::Error::custom(format!(
                        "unsupported measurement type: {other:?}"
                    ))),
                }
            }
        }

        deserializer.deserialize_any(MeasurementValueVisitor)
    }
}
