use serde::Deserialize ;
use super::FloatSemantic;

#[derive(Debug, Clone)]
pub enum Value {
    Boolean(BooleanValue),
    Integer(NumericValue),
    Float(FloatValue),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanValue {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericValue {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
    /// Which statistical aggregates should be tracked for this value
    /// over time (e.g. running min/max). Optional.
    /// Default is no statistics tracked.
    #[serde(default)]
    pub statistics: NumericValueStatistics,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FloatValue {
    /// Semantic Representation of the float. E.g. plain, fraction, millimeter
    pub semantic: FloatSemantic,
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
    /// Which statistical aggregates should be tracked for this value
    /// over time (e.g. running min/max). Optional.
    /// Default is no statistics tracked.
    #[serde(default)]
    pub statistics: NumericValueStatistics,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericValueStatistics {
    /// Track the minimum observed value each sampling cycle. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub min: bool,
    /// Track the minimum observed value each sampling cycle. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub max: bool,
}
