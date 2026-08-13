use core::fmt;
use indexmap::IndexMap;

pub enum CommandArgField {
    Unset,
    Null,
    Object(IndexMap<String, CommandArgField>),
    Array(Vec<CommandArgField>),
    Enum(String),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl fmt::Display for CommandArgField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandArgField::Unset => write!(f, "<unset>"),
            CommandArgField::Null => write!(f, "null"),
            CommandArgField::Object(fields) => {
                write!(f, "{{{} fields}}", fields.len())
            }
            CommandArgField::Array(values) => {
                write!(f, "[{} items]", values.len())
            }
            CommandArgField::Enum(value) => write!(f, "{value}"),
            CommandArgField::String(value) => write!(f, "{value}"),
            CommandArgField::Boolean(value) => write!(f, "{value}"),
            CommandArgField::Integer(value) => write!(f, "{value}"),
            CommandArgField::Float(value) => write!(f, "{value:.3}"),
        }
    }
}
