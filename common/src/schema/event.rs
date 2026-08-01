use super::EnumVariants;
use super::FieldMetadata;
use super::FloatSemantic;
use super::StringMap;
use super::Type;

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub fields: StringMap<EventField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventField {
    pub kind: EventFieldKind,
    pub nullable: bool,
    pub metadata: FieldMetadata,
}

#[derive(Debug, Clone, Serialize)]
pub enum EventFieldKind {
    Object { fields: StringMap<EventField> },
    Array { item: Box<EventField> },
    Enum { variants: EnumVariants },
    String,
    Boolean,
    Integer,
    Float { semantic: FloatSemantic },
}

// --- deserialize implemenations ---
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = Event;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let ty = Type::from_str(&tag).map_err(A::Error::custom)?;

                if !matches!(ty, Type::Event) {
                    return Err(Error::custom(format!("expected !event, received: !{tag}.")));
                }

                let fields = variant.newtype_variant::<StringMap<EventField>>()?;

                Ok(Event { fields })
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("!event or object")
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}

impl<'de> Deserialize<'de> for EventField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = EventField;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let ty = Type::from_str(&tag).map_err(A::Error::custom)?;

                match ty {
                    Type::Object => {
                        let ObjectHelper { fields, nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Object { fields },
                            metadata: Default::default(),
                        })
                    }

                    Type::Array => {
                        let ArrayHelper { item, nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Array { item },
                            metadata: Default::default(),
                        })
                    }
                    Type::Enum => {
                        let EnumHelper { variants, nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Enum { variants },
                            metadata: Default::default(),
                        })
                    }
                    Type::String => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::String,
                            metadata: Default::default(),
                        })
                    }
                    Type::Boolean => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Boolean,
                            metadata: Default::default(),
                        })
                    }
                    Type::Integer => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Integer,
                            metadata: Default::default(),
                        })
                    }
                    Type::Float(semantic) => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Float { semantic },
                            metadata: Default::default(),
                        })
                    }

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
    fields: StringMap<EventField>,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrayHelper {
    item: Box<EventField>,

    #[serde(default)]
    nullable: bool,
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
struct SimpleHelper {
    #[serde(default)]
    nullable: bool,
}
