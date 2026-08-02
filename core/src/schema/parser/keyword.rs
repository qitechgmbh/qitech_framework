use std::str::FromStr;

use crate::schema::FloatSemantic;

#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Object,
    Array,
    Enum,
    String,
    Boolean,
    Integer,
    Float(FloatSemantic),
    Command,
    Event,
}

impl FromStr for Keyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = FloatSemantic::from_str(s) {
            return Ok(Self::Float(v));
        }

        Ok(match s {
            "command" => Self::Command,
            "event" => Self::Event,
            "object" => Self::Object,
            "array" => Self::Array,
            "enum" => Self::Enum,
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            other => return Err(format!("invalid type {other}")),
        })
    }
}
