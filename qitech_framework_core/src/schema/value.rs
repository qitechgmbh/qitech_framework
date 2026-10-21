use core::fmt;

use serde::Deserialize;
use serde::Serialize;

// --- value definition ---
/// A single scalar leaf, e.g. `limit_inner float quantity=si.length`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValueDefinition {
    /// Dotted path relative to its section, e.g. `traverse.limit_inner`.
    pub path: String,
    pub ty: ValueType,
    pub optional: bool,
}

// --- value type ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValueType {
    Boolean,
    Integer { quantity: Option<QuantityRef> },
    Float { quantity: Option<QuantityRef> },
    String { max_len: Option<usize> },
    Enum { variants: Vec<EnumVariant> },
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "boolean"),
            Self::Integer { quantity } => fmt_numeric(f, "integer", quantity),
            Self::Float { quantity } => fmt_numeric(f, "float", quantity),
            Self::String { .. } => write!(f, "string"),
            Self::Enum { variants } => {
                write!(f, "enum(")?;
                for (i, variant) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", variant.name)?;
                }
                write!(f, ")")
            }
        }
    }
}

fn fmt_numeric(
    f: &mut fmt::Formatter<'_>,
    name: &str,
    quantity: &Option<QuantityRef>,
) -> fmt::Result {
    match quantity {
        Some(quantity) => write!(f, "{name}({quantity})"),
        None => write!(f, "{name}"),
    }
}

// --- quantity reference ---
/// Reference to a quantity of an imported unit catalog, e.g. `si.length`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuantityRef {
    pub import: String,
    pub quantity: String,
}

impl fmt::Display for QuantityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.import, self.quantity)
    }
}

// --- enum variant ---
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    /// Stable integer value; implicit variants are numbered in declaration order.
    pub value: i64,
}
