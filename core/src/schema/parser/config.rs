use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::ConfigPropertyDefinition;
use crate::schema::ConfigPropertyKind;
use crate::schema::FloatSemantic;
use crate::schema::parser::enum_variants::EnumVariantsRaw;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct ConfigPropertyInfoRaw(pub ConfigPropertyDefinition);

impl<'de> Deserialize<'de> for ConfigPropertyInfoRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigPropertyVisitor;

        impl<'de> Visitor<'de> for ConfigPropertyVisitor {
            type Value = ConfigPropertyInfoRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a tagged config property value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Keyword::from_str(&tag).map_err(A::Error::custom)? {
                    Keyword::Enum => {
                        let helper = variant.newtype_variant()?;
                        process_enum(helper)
                    }

                    Keyword::String => {
                        let helper = variant.newtype_variant()?;
                        process_string(helper)
                    }

                    Keyword::Boolean => {
                        let helper = variant.newtype_variant()?;
                        process_bool(helper)
                    }

                    Keyword::Integer => {
                        let helper = variant.newtype_variant()?;
                        process_integer(helper)
                    }

                    Keyword::Float(semantic) => {
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
struct EnumValueHelper {
    #[serde(default)]
    nullable: bool,

    #[serde(default = "persistent_default")]
    persistent: bool,

    variants: EnumVariantsRaw,

    #[serde(default)]
    default: Option<String>,
}

fn process_enum<E: Error>(helper: EnumValueHelper) -> Result<ConfigPropertyInfoRaw, E> {
    let EnumValueHelper {
        nullable,
        persistent,
        variants,
        default,
    } = helper;

    let variants = variants.0;

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

    Ok(ConfigPropertyInfoRaw(ConfigPropertyDefinition {
        kind: ConfigPropertyKind::Enum { variants, default },
        nullable,
        persistent,
        metadata: Default::default(),
    }))
}

// --- string ---
fn process_string<E: Error>(helper: SimpleValueHelper<String>) -> Result<ConfigPropertyInfoRaw, E> {
    process_simple(helper, |default| ConfigPropertyKind::String { default })
}

// --- boolean ---
fn process_bool<E: Error>(helper: SimpleValueHelper<bool>) -> Result<ConfigPropertyInfoRaw, E> {
    process_simple(helper, |default| ConfigPropertyKind::Boolean { default })
}

// --- integer ---
fn process_integer<E: Error>(helper: SimpleValueHelper<i64>) -> Result<ConfigPropertyInfoRaw, E> {
    process_simple(helper, |default| ConfigPropertyKind::Integer { default })
}

// --- float ---
fn process_float<E: Error>(
    helper: SimpleValueHelper<f64>,
    semantic: FloatSemantic,
) -> Result<ConfigPropertyInfoRaw, E> {
    process_simple(helper, move |default| ConfigPropertyKind::Float {
        semantic,
        default,
    })
}

// --- helpers ---
fn process_simple<T, E, F>(
    helper: SimpleValueHelper<T>,
    build_kind: F,
) -> Result<ConfigPropertyInfoRaw, E>
where
    T: FromStr,
    T::Err: Display,
    E: Error,
    F: FnOnce(Option<T>) -> ConfigPropertyKind,
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

    Ok(ConfigPropertyInfoRaw(ConfigPropertyDefinition {
        kind: build_kind(default),
        nullable,
        persistent,
        metadata: Default::default(),
    }))
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
