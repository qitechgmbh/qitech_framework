use crate::machine_schema::r#type::{self, Type};

use super::{EnumVariants, Range, FloatSemantic};

#[derive(Debug, Clone)]
pub struct ConfigPropertyValue {
    pub kind: ConfigPropertyValueKind,
    pub nullable: bool,
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub enum ConfigPropertyValueKind {
    Enum {
        /// Variants of the enum. Required.
        variants: EnumVariants,
        /// Default enum variant. Required.
        default: Option<String>,
    },
    String {
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<String>,
        /// Character length bounds. Optional.
        /// Default is `Unbounded`.
        bounds: Range<u32>,
    },
    Boolean {
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<bool>,
    },
    Integer {
        default: Option<i64>,
        /// Allowed range for this value. Optional.
        /// Default is unbounded.
        range: Range<i64>,
    },
    Float {
        /// Representation of the float. E.g. plain, fraction, millimeter
        semantic: FloatSemantic,
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<f64>,
        /// Allowed range for this value. Optional.
        /// Default is unbounded.
        range: Range<f64>,
    },
}

// --- deserialize implemenations ---
use std::str::FromStr;
use std::fmt::{self, Display};
use serde::{Deserialize, de::{Error, Deserializer, DeserializeOwned, Visitor, EnumAccess}};
use serde::de::value::{EnumAccessDeserializer};

impl<'de> Deserialize<'de> for ConfigPropertyValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigPropertyVisitor;

        impl<'de> Visitor<'de> for ConfigPropertyVisitor {
            type Value = ConfigPropertyValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged config property value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let tag = tag
                    .strip_prefix('!')
                    .ok_or_else(|| A::Error::custom("expected yaml tag"))?;

                let ty = r#type::parse(tag)
                    .map_err(A::Error::custom)?;

                match ty {
                    Type::Enum => {
                        let helper =
                            EnumValueHelper::deserialize(
                                EnumAccessDeserializer::new(data)
                            )?;

                        process_enum(helper)
                    }

                    Type::String => {
                        let helper =
                            StringValueHelper::deserialize(
                                EnumAccessDeserializer::new(data)
                            )?;

                        process_string(helper)
                    }

                    Type::Boolean => {
                        let helper =
                            BooleanValueHelper::deserialize(
                                EnumAccessDeserializer::new(data)
                            )?;

                        process_bool(helper)
                    }

                    Type::Integer => {
                        let helper =
                            NumericValueHelper::deserialize(
                                EnumAccessDeserializer::new(data)
                            )?;

                        process_integer(helper)
                    }

                    Type::Float(semantic) => {
                        let helper =
                            NumericValueHelper::deserialize(
                                EnumAccessDeserializer::new(data)
                            )?;

                        process_float(helper, semantic)
                    }

                    other => {
                        Err(A::Error::custom(format!(
                            "unsupported config property type: {other:?}"
                        )))
                    }
                }
            }
        }

        deserializer.deserialize_any(ConfigPropertyVisitor)
    }
}

// --- enum ---
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumValueHelper {
    #[serde(default)]
    nullable: bool,

    #[serde(default = "persistent_default")]
    persistent: bool,

    variants: EnumVariants,

    #[serde(default)]
    default: Option<String>,
}

fn process_enum<E: Error>(helper: EnumValueHelper) -> Result<ConfigPropertyValue, E> {
    let EnumValueHelper {
        nullable,
        persistent,
        variants,
        default,
    } = helper;

    if !nullable {
        let Some(variant) = &default else {
            return Err(E::custom(
                "`default` is required when `nullable` is `false`",
            ));
        };

        if variants.get_int(variant).is_none() {
            return Err(E::custom(format!("no variant named '{}'", variant)));
        }
    }

    Ok(ConfigPropertyValue {
        kind: ConfigPropertyValueKind::Enum { variants, default }, 
        nullable, 
        persistent 
    })
}

// --- string ---
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StringValueHelper {
    #[serde(default)]
    default: Option<String>,
    #[serde(default)]
    bounds: Range<u32>,
    #[serde(default)]
    nullable: bool,
    #[serde(default = "persistent_default")]
    persistent: bool,
}

fn process_string<E: Error>(helper: StringValueHelper) -> Result<ConfigPropertyValue, E> {
    let StringValueHelper {
        default,
        bounds,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    // ensure default itself lies in the provided range
    if let Some(v) = &default {
        let len = v.len() as u32;
        if !bounds.in_range(len) {
            return Err(Error::custom(format!(
                "default value '{v}' (len = {len}) is outside the allowed length range {bounds}"
            )));
        }
    }

    Ok(ConfigPropertyValue {
        kind: ConfigPropertyValueKind::String { default, bounds }, 
        nullable, 
        persistent 
    })
}

// --- boolean ---
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanValueHelper {
    #[serde(default)]
    pub default: Option<bool>,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default = "persistent_default")]
    pub persistent: bool,
}

fn process_bool<E: Error>(helper: BooleanValueHelper) -> Result<ConfigPropertyValue, E> {
    let BooleanValueHelper {
        default,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(ConfigPropertyValue {
        kind: ConfigPropertyValueKind::Boolean { default }, 
        nullable, 
        persistent 
    })
}

// --- integer ---
fn process_integer<E: Error>(helper: NumericValueHelper<i64>) -> Result<ConfigPropertyValue, E> {
    let NumericValueHelper {
        default,
        range,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(ConfigPropertyValue {
        kind: ConfigPropertyValueKind::Integer { default, range }, 
        nullable, 
        persistent 
    })
}

// --- float ---
fn process_float<E: Error>(
    helper: NumericValueHelper<f64>,
    semantic: FloatSemantic,
) -> Result<ConfigPropertyValue, E> {
    let NumericValueHelper {
        default,
        range,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(ConfigPropertyValue {
        kind: ConfigPropertyValueKind::Float { semantic, default, range }, 
        nullable, 
        persistent 
    })
}

// --- helpers ---
#[derive(Debug, Clone, Deserialize)]
#[serde(
    deny_unknown_fields,
    bound(deserialize = "T: DeserializeOwned")
)]
struct NumericValueHelper<T>
where
    T: FromStr,
    T::Err: Display,
{
    #[serde(default)]
    pub nullable: bool,

    #[serde(default)]
    pub default: Option<T>,

    #[serde(default)]
    pub range: Range<T>,

    #[serde(default = "persistent_default")]
    pub persistent: bool,
}

fn persistent_default() -> bool { true }
