use std::fmt::Display;
use std::str::FromStr;

use crate::machine_schema::{FieldMetadata, Range, Type, r#type};
use crate::machine_schema::{EnumVariants, FloatSemantic, types::StringMap};

#[derive(Debug, Clone)]
pub struct Command {
    pub fields: StringMap<CommandField>,
}

#[derive(Debug, Clone)]
pub struct CommandField {
    pub kind: CommandFieldKind,
    pub nullable: bool,
    pub metadata: FieldMetadata,
}

#[derive(Debug, Clone)]
pub enum CommandFieldKind {
    Object {
        fields: StringMap<CommandField>,
    },
    Array {
        item: Box<CommandField>,
        bounds: Range<u32>,
    },
    Enum {
        variants: EnumVariants,
    },
    String {
        /// regex pattern to validate input
        pattern: Option<String>,

        /// character length bounds
        bounds: Range<u32>,
    },
    Boolean,
    Integer {
        /// value bounds
        bounds: Option<Range<i64>>,
    },
    Float {
        semantic: FloatSemantic,

        /// value bounds
        bounds: Option<Range<f64>>,
    },
}

// --- deserialize implemenations ---
use serde::Deserialize;
use serde::de::{Deserializer, EnumAccess, Error, VariantAccess, Visitor};

impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = Command;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let value_type = r#type::parse(&tag)
                    .map_err(A::Error::custom)?;

                if !matches!(value_type, Type::Command) {
                    return Err(Error::custom(format!("expected !command, received: !{tag}.")))
                }

                let fields = variant.newtype_variant::<StringMap<CommandField>>()?;

                Ok(Command { fields })
            }
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("!event or object")
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}

impl<'de> Deserialize<'de> for CommandField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = CommandField;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let value_type = r#type::parse(&tag)
                    .map_err(A::Error::custom)?;

                match value_type {
                    Type::Object => {
                        let ObjectHelper { fields, nullable } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Object { fields },
                            metadata: Default::default(),
                        })
                    },

                    Type::Array => {
                        let ArrayHelper { item, nullable, bounds } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Array { item, bounds },
                            metadata: Default::default(),
                        })
                    },
                    Type::Enum => {
                        let EnumHelper { variants, nullable } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Enum { variants },
                            metadata: Default::default(),
                        })
                    },
                    Type::String => {
                        let StringHelper { nullable, bounds, pattern } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::String { pattern, bounds },
                            metadata: Default::default(),
                        })
                    },
                    Type::Boolean => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Boolean,
                            metadata: Default::default(),
                        })
                    },
                    Type::Integer => {
                        let NumericHelper { nullable, bounds } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Integer { bounds },
                            metadata: Default::default(),
                        })
                    },
                    Type::Float(semantic) => {
                        let NumericHelper { nullable, bounds } = variant.newtype_variant()?;

                        Ok(CommandField {
                            nullable,
                            kind: CommandFieldKind::Float { semantic, bounds },
                            metadata: Default::default(),
                        })
                    },

                    other => Err(Error::custom(format!("Unsupported type: {other:?}"))),
                }
            }
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("regular value")
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}

// --- enum ---
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectHelper {
    fields: StringMap<CommandField>,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrayHelper {
    item: Box<CommandField>,

    #[serde(default)]
    nullable: bool,

    #[serde(default)]
    bounds: Range<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumHelper {
    variants: EnumVariants,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StringHelper {
    #[serde(default)]
    nullable: bool,

    #[serde(default)]
    bounds: Range<u32>,

    #[serde(default)]
    pattern: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct NumericHelper<T>
where 
    T: FromStr,
    T::Err: Display
{
    #[serde(default)]
    nullable: bool,

    #[serde(default)]
    bounds: Option<Range<T>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimpleHelper {
    #[serde(default)]
    nullable: bool,
}
