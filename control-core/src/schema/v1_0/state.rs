use serde::Deserialize ;
use super::{EnumVariants, FloatSemantic};

#[derive(Debug, Clone)]
pub enum Value {
    Enum(EnumValue),
    String(ScalarValue),
    Boolean(ScalarValue),
    Integer(ScalarValue),
    Float(FloatValue),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumValue {
    /// The set of allowed variants for this value. Required.
    pub variants: EnumVariants,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarValue {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloatValue {
    /// Representation of the float. E.g. plain, fraction, millimeter
    pub semantic: FloatSemantic,

    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
}
