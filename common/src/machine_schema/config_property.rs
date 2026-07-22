use super::{EnumVariants, Range, FloatSemantic};

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub nullable: bool,
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub enum ValueKind {
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
use std::fmt::Display;
use serde::{Deserialize, de::{self, Deserializer, DeserializeOwned}};
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
                let helper = EnumValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                process_enum(helper)
            },
            ValueType::String => {
                let helper = StringValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                process_string(helper)
            }
            ValueType::Boolean => {
                let helper = BooleanValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                process_bool(helper)
            }
            ValueType::Integer => {
                let helper = NumericValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                process_integer(helper)
            }
            ValueType::Float(semantic) => {
                let helper = NumericValueHelper::deserialize(value)
                    .map_err(de::Error::custom)?;

                process_float(helper, semantic)
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

    #[serde(default = "persistent_default")]
    persistent: bool,

    variants: EnumVariants,

    #[serde(default)]
    default: Option<String>,
}

fn process_enum<E: de::Error>(helper: EnumValueHelper) -> Result<Value, E> {
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
            return Err(E::custom(format!("no such variant {:?}", variant)));
        }
    }

    Ok(Value {
        kind: ValueKind::Enum { variants, default }, 
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

fn process_string<E: de::Error>(helper: StringValueHelper) -> Result<Value, E> {
    let StringValueHelper {
        default,
        bounds,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(de::Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    // ensure default itself lies in the provided range
    if let Some(v) = &default {
        let len = v.len() as u32;
        if !bounds.in_range(len) {
            return Err(de::Error::custom(format!(
                "default value '{v}' (len = {len}) is outside the allowed length range {bounds}"
            )));
        }
    }

    Ok(Value {
        kind: ValueKind::String { default, bounds }, 
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

fn process_bool<E: de::Error>(helper: BooleanValueHelper) -> Result<Value, E> {
    let BooleanValueHelper {
        default,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(de::Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(Value {
        kind: ValueKind::Boolean { default }, 
        nullable, 
        persistent 
    })
}

// --- integer ---
fn process_integer<E: de::Error>(helper: NumericValueHelper<i64>) -> Result<Value, E> {
    let NumericValueHelper {
        default,
        range,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(de::Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(Value {
        kind: ValueKind::Integer { default, range }, 
        nullable, 
        persistent 
    })
}

// --- float ---
fn process_float<E: de::Error>(
    helper: NumericValueHelper<f64>,
    semantic: FloatSemantic,
) -> Result<Value, E> {
    let NumericValueHelper {
        default,
        range,
        nullable,
        persistent,
    } = helper;

    if !nullable && default.is_none() {
        return Err(de::Error::custom(
            "`default` is required when `nullable` is `false`",
        ));
    }

    Ok(Value {
        kind: ValueKind::Float { semantic, default, range }, 
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
