use std::str::FromStr;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::EventDefinition;
use crate::schema::EventField;
use crate::schema::EventFieldKind;
use crate::schema::StringMap;
use crate::schema::parser::enum_variants::EnumVariantsRaw;
use crate::schema::parser::keyword::Keyword;

#[derive(Debug)]
pub struct EventInfoRaw(pub EventDefinition);

impl<'de> Deserialize<'de> for EventInfoRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = EventInfoRaw;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let ty = Keyword::from_str(&tag).map_err(A::Error::custom)?;

                if !matches!(ty, Keyword::Event) {
                    return Err(Error::custom(format!("expected !event, received: !{tag}.")));
                }

                let fields_raw = variant.newtype_variant::<StringMap<EventInfoFieldRaw>>()?;

                // Unwrap each EventInfoFieldRaw -> EventField
                let fields: StringMap<EventField> = fields_raw
                    .into_iter()
                    .map(|(key, EventInfoFieldRaw(field))| (key, field))
                    .collect();

                Ok(EventInfoRaw(EventDefinition {
                    fields,
                    metadata: Default::default(),
                }))
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("!event or object")
            }
        }

        deserializer.deserialize_any(MyVisitor)
    }
}

// --- field ---
#[derive(Debug)]
pub struct EventInfoFieldRaw(pub EventField);

impl<'de> Deserialize<'de> for EventInfoFieldRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyVisitor;

        impl<'de> Visitor<'de> for MyVisitor {
            type Value = EventInfoFieldRaw;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let ty = Keyword::from_str(&tag).map_err(A::Error::custom)?;

                match ty {
                    Keyword::Object => {
                        let ObjectHelper { fields, nullable } = variant.newtype_variant()?;

                        let fields: StringMap<EventField> = fields
                            .into_iter()
                            .map(|(key, EventInfoFieldRaw(field))| (key, field))
                            .collect();

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Object { fields },
                        }))
                    }

                    Keyword::Array => {
                        let ArrayHelper { item, nullable } = variant.newtype_variant()?;

                        let EventInfoFieldRaw(item) = *item;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Array {
                                item: Box::new(item),
                            },
                        }))
                    }

                    Keyword::Enum => {
                        let EnumHelper { variants, nullable } = variant.newtype_variant()?;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Enum {
                                variants: variants.0,
                            },
                        }))
                    }

                    Keyword::String => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::String,
                        }))
                    }

                    Keyword::Boolean => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Boolean,
                        }))
                    }

                    Keyword::Integer => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Integer,
                        }))
                    }

                    Keyword::Float(semantic) => {
                        let SimpleHelper { nullable } = variant.newtype_variant()?;

                        Ok(EventInfoFieldRaw(EventField {
                            nullable,
                            kind: EventFieldKind::Float { semantic },
                        }))
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
    fields: StringMap<EventInfoFieldRaw>,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrayHelper {
    item: Box<EventInfoFieldRaw>,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnumHelper {
    variants: EnumVariantsRaw,

    #[serde(default)]
    nullable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SimpleHelper {
    #[serde(default)]
    nullable: bool,
}
