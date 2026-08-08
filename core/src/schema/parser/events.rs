use core::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::de;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::Error;
use serde::de::VariantAccess;
use serde::de::Visitor;

use crate::schema::EventDefinition;
use crate::schema::EventField;
use crate::schema::EventFieldKind;
use crate::schema::FloatSemantic;
use crate::schema::StringMap;
use crate::schema::parser::enum_variants::EnumVariantsRaw;

#[derive(Debug)]
pub struct EventDefinitionRaw(pub EventDefinition);

impl<'de> Deserialize<'de> for EventDefinitionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> Visitor<'de> for EventVisitor {
            type Value = EventDefinition;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                if tag != "event" {
                    return Err(de::Error::custom(format!(
                        "expected !event, received: !{tag}."
                    )));
                }

                let fields_raw = variant.newtype_variant::<StringMap<EventFieldRaw>>()?;

                // Unwrap each EventInfoFieldRaw -> EventField
                let fields: StringMap<EventField> = fields_raw
                    .into_iter()
                    .map(|(key, EventFieldRaw(field))| (key, field))
                    .collect();

                Ok(EventDefinition {
                    fields,
                    metadata: Default::default(),
                })
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("event")
            }
        }

        let definition = deserializer.deserialize_any(EventVisitor)?;
        Ok(EventDefinitionRaw(definition))
    }
}

// --- field ---
#[derive(Debug)]
pub struct EventFieldRaw(pub EventField);

impl<'de> Deserialize<'de> for EventFieldRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = EventField;

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let (tag, variant) = data.variant::<String>()?;

                let nullable = tag.starts_with("?");
                let tag = if nullable { &tag[1..] } else { &tag };

                if let Ok(semantic) = FloatSemantic::from_str(tag) {
                    variant.unit_variant()?;

                    return Ok(EventField {
                        kind: EventFieldKind::Float { semantic },
                        nullable,
                    });
                }

                match tag {
                    "object" => {
                        let fields = variant.newtype_variant::<StringMap<EventFieldRaw>>()?;

                        // Unwrap each EventInfoFieldRaw -> EventField
                        let fields: StringMap<EventField> = fields
                            .into_iter()
                            .map(|(key, EventFieldRaw(field))| (key, field))
                            .collect();

                        Ok(EventField {
                            nullable,
                            kind: EventFieldKind::Object { fields },
                        })
                    }

                    "list" => {
                        #[derive(Deserialize)]
                        #[serde(deny_unknown_fields)]
                        struct ListHelper {
                            item: EventFieldRaw,
                        }

                        let ListHelper { item } = variant.newtype_variant()?;

                        Ok(EventField {
                            kind: EventFieldKind::List {
                                item: Box::new(item.0),
                            },
                            nullable,
                        })
                    }

                    "enum" => {
                        let EnumVariantsRaw(variants) = variant.newtype_variant()?;

                        Ok(EventField {
                            kind: EventFieldKind::Enum { variants },
                            nullable,
                        })
                    }

                    "string" => {
                        variant.unit_variant()?;

                        Ok(EventField {
                            kind: EventFieldKind::String,
                            nullable,
                        })
                    }

                    "boolean" => {
                        variant.unit_variant()?;

                        Ok(EventField {
                            kind: EventFieldKind::Boolean,
                            nullable,
                        })
                    }

                    "integer" => {
                        variant.unit_variant()?;

                        Ok(EventField {
                            kind: EventFieldKind::Integer,
                            nullable,
                        })
                    }

                    other => Err(A::Error::custom(format!(
                        "unsupported event field type: {other:?}"
                    ))),
                }
            }

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("regular value")
            }
        }

        let field = deserializer.deserialize_any(FieldVisitor)?;
        Ok(EventFieldRaw(field))
    }
}
