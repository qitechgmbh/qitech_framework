use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;

use crate::schema::ConfigPropertyDefinition;
use crate::schema::ConfigPropertyKind;
use crate::schema::parser::enum_variants::EnumVariantsRaw;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct ConfigPropertyInfoRaw(pub ConfigPropertyDefinition);

impl<'de> Deserialize<'de> for ConfigPropertyInfoRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ConfigPropertyInfoRaw;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a tagged config property value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                match Keyword::from_str(&tag).map_err(A::Error::custom)? {
                    Keyword::Enum => process_enum(variant.newtype_variant()?),

                    Keyword::String => {
                        process_simple(variant.newtype_variant()?, ConfigPropertyKind::String)
                    }

                    Keyword::Boolean => {
                        process_simple(variant.newtype_variant()?, ConfigPropertyKind::Boolean)
                    }

                    Keyword::Integer => {
                        process_simple(variant.newtype_variant()?, ConfigPropertyKind::Integer)
                    }

                    Keyword::Float(semantic) => process_simple(
                        variant.newtype_variant()?,
                        ConfigPropertyKind::Float { semantic },
                    ),

                    other => Err(A::Error::custom(format!(
                        "unsupported config property type: {other:?}"
                    ))),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
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
        let Some(default) = &default else {
            return Err(E::custom("`default` is required when `nullable` is false"));
        };

        if variants.get_int(default).is_none() {
            return Err(E::custom(format!("no variant named '{}'", default)));
        }
    }

    Ok(ConfigPropertyInfoRaw(ConfigPropertyDefinition {
        kind: ConfigPropertyKind::Enum { variants },
        nullable,
        persistent,
        metadata: Default::default(),
    }))
}

// --- simple ---
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimpleValueHelper {
    #[serde(default)]
    nullable: bool,

    #[serde(default = "persistent_default")]
    persistent: bool,
}

fn process_simple<E: Error>(
    helper: SimpleValueHelper,
    kind: ConfigPropertyKind,
) -> Result<ConfigPropertyInfoRaw, E> {
    Ok(ConfigPropertyInfoRaw(ConfigPropertyDefinition {
        kind,
        nullable: helper.nullable,
        persistent: helper.persistent,
        metadata: Default::default(),
    }))
}

// --- utils ---
fn persistent_default() -> bool {
    true
}
