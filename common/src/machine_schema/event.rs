use crate::machine_schema::{ValueType, value_type};
use crate::machine_schema::{EnumVariants, FloatSemantic, types::StringMap};

#[derive(Debug, Clone)]
pub struct EventValue {
    pub kind: EventValueKind,
    pub nullable: bool,
}

#[derive(Debug, Clone)]
pub enum EventValueKind {
    Group {
        items: StringMap<EventValue>,
    },
    Array {
        item: Box<EventValue>
    },
    Enum {
        variants: EnumVariants,
    },
    Boolean,
    Integer,
    Float {
        semantic: FloatSemantic,
    },
}

// --- deserialize implemenations ---
use serde::Deserialize;
use serde::de::{self, Deserializer};

impl<'de> Deserialize<'de> for EventValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = yaml_serde::Value::deserialize(deserializer)?;

        let yaml_serde::Value::Tagged(tagged) = value else {
            return Err(de::Error::custom("expected tagged value"));
        };

        // skip te '!'
        let tag = &tagged.tag.to_string()[1..];

        // read the value type / tag
        let value_t = value_type::parse(tag)
            .map_err(de::Error::custom)?;
        
        let value = tagged.value;
    }
}
