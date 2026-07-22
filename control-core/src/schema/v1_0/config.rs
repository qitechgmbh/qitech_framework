use super::{EnumVariants, Range, FloatSemantic};

#[derive(Debug, Clone)]
pub enum Value {
    Enum(EnumValue),
    String(StringValue),
    Boolean(BooleanValue),
    Integer(IntegerValue),
    Float(FloatValue),
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    /// Variants of the enum. Required.
    pub variants: EnumVariants,
    /// Default enum variant. Required.
    pub default: String,
    /// Whether this property should be included in config exports. Optional.
    /// Default is `true`.
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub struct StringValue {
    /// Whether this value is allowed to be null. If `true`, `default` becomes optional.
    /// Default is `false`.
    pub nullable: bool,
    /// Default value. Required if `nullable` is `false`.
    /// Default is `None` if `nullable` is `true`.
    pub default: Option<String>,
    /// Character length bounds. Optional.
    /// Default is `Unbounded`.
    pub length: Range<u32>,
    /// Whether this property should be included in config exports. Optional.
    /// Default is `true`.
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub struct BooleanValue {
    /// Whether this value is allowed to be null. If `true`, `default` becomes optional.
    /// Default is `false`.
    pub nullable: bool,
    /// Default value. Required if `nullable` is `false`.
    /// Default is `None` if `nullable` is `true`.
    pub default: Option<bool>,
    /// Whether this property should be included in config exports. Optional.
    /// Default is `true`.
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub struct IntegerValue {
    /// Whether this value is allowed to be null. If `true`, `default` becomes optional.
    /// Default is `false`.
    pub nullable: bool,
    /// Default value. Required if `nullable` is `false`.
    /// Default is `None` if `nullable` is `true`.
    pub default: Option<i64>,
    /// Allowed range for this value. Optional.
    /// Default is unbounded.
    pub range: Range<i64>,
    /// Whether this property should be included in config exports. Optional.
    /// Default is `true`.
    pub persistent: bool,
}

#[derive(Debug, Clone)]
pub struct FloatValue {
    /// Representation of the float. E.g. plain, fraction, millimeter
    pub semantic: FloatSemantic,
    /// Whether this value is allowed to be null. If `true`, `default` becomes optional.
    /// Default is `false`.
    pub nullable: bool,
    /// Default value. Required if `nullable` is `false`.
    /// Default is `None` if `nullable` is `true`.
    pub default: Option<f64>,
    /// Allowed range for this value. Optional.
    /// Default is unbounded.
    pub range: Range<f64>,
    /// Whether this property should be included in config exports. Optional.
    /// Default is `true`.
    pub persistent: bool,
}
