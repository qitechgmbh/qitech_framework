use super::EnumVariants;
use super::FloatSemantic;

#[derive(Debug, Clone, Serialize)]
pub struct ConfigPropertyValue {
    pub kind: ConfigPropertyValueKind,
    pub nullable: bool,
    pub persistent: bool,
}

#[derive(Debug, Clone, Serialize)]
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
    },
    Boolean {
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<bool>,
    },
    Integer {
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<i64>,
    },
    Float {
        /// Representation of the float. E.g. plain, fraction, millimeter
        semantic: FloatSemantic,
        /// Default value. Required if `nullable` is `false`.
        /// Default is `None` if `nullable` is `true`.
        default: Option<f64>,
    },
}

// --- deserialize implemenations ---
use std::fmt::Display;
use std::fmt::{self};
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use super::Type;

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

                match Type::from_str(&tag).map_err(A::Error::custom)? {
                    Type::Enum => {
                        let helper = variant.newtype_variant()?;
                        process_enum(helper)
                    }

                    Type::String => {
                        let helper = variant.newtype_variant()?;
                        process_string(helper)
                    }

                    Type::Boolean => {
                        let helper = variant.newtype_variant()?;
                        process_bool(helper)
                    }

                    Type::Integer => {
                        let helper = variant.newtype_variant()?;
                        process_integer(helper)
                    }

                    Type::Float(semantic) => {
                        let helper = variant.newtype_variant()?;
                        process_float(helper, semantic)
                    }

                    other => Err(A::Error::custom(format!(
                        "unsupported config property type: {other:?}"
                    ))),
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
        persistent,
    })
}

// --- string ---
fn process_string<E: Error>(helper: SimpleValueHelper<String>) -> Result<ConfigPropertyValue, E> {
    process_simple(helper, |default| ConfigPropertyValueKind::String {
        default,
    })
}

// --- boolean ---
fn process_bool<E: Error>(helper: SimpleValueHelper<bool>) -> Result<ConfigPropertyValue, E> {
    process_simple(helper, |default| ConfigPropertyValueKind::Boolean {
        default,
    })
}

// --- integer ---
fn process_integer<E: Error>(helper: SimpleValueHelper<i64>) -> Result<ConfigPropertyValue, E> {
    process_simple(helper, |default| ConfigPropertyValueKind::Integer {
        default,
    })
}

// --- float ---
fn process_float<E: Error>(
    helper: SimpleValueHelper<f64>,
    semantic: FloatSemantic,
) -> Result<ConfigPropertyValue, E> {
    process_simple(helper, move |default| ConfigPropertyValueKind::Float {
        semantic,
        default,
    })
}

// --- helpers ---
fn process_simple<T, E, F>(
    helper: SimpleValueHelper<T>,
    build_kind: F,
) -> Result<ConfigPropertyValue, E>
where
    T: FromStr,
    T::Err: Display,
    E: Error,
    F: FnOnce(Option<T>) -> ConfigPropertyValueKind,
{
    let SimpleValueHelper {
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
        kind: build_kind(default),
        nullable,
        persistent,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, bound(deserialize = "T: DeserializeOwned"))]
struct SimpleValueHelper<T>
where
    T: FromStr,
    T::Err: Display,
{
    #[serde(default)]
    pub nullable: bool,

    #[serde(default)]
    pub default: Option<T>,

    #[serde(default = "persistent_default")]
    pub persistent: bool,
}

fn persistent_default() -> bool {
    true
}
