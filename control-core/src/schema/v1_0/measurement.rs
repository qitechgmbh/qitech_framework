use serde::Deserialize ;
use super::{EnumVariants, Unit};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Enum(EnumValue),
    Boolean(BooleanValue),
    Integer(NumericValue),
    Float(NumericValue),
    Fraction(NumericValue),
    Percentage(NumericValue),

    // Acceleration
    MeterPerSecondSquared(NumericValue),
    MeterPerMinutePerSecond(NumericValue),

    // AmountOfSubstance
    Mole(NumericValue),

    // Angle
    Radian(NumericValue),
    Degree(NumericValue),
    Revolution(NumericValue),

    // AngularAcceleration
    RadianPerSecondSquared(NumericValue),
    DegreePerSecondSquared(NumericValue),
    RevolutionPerMinutePerSecond(NumericValue),

    // AngularJerk
    RadianPerSecondCubed(NumericValue),
    DegreePerSecondCubed(NumericValue),
    RevolutionPerMinutePerSecondSquared(NumericValue),

    // AngularVelocity
    RadianPerSecond(NumericValue),
    DegreePerSecond(NumericValue),
    RevolutionPerSecond(NumericValue),
    RevolutionPerMinute(NumericValue),

    // ElectricCurrent
    Milliampere(NumericValue),
    Centiampere(NumericValue),
    Ampere(NumericValue),

    // ElectricPotential
    Millivolt(NumericValue),
    Centivolt(NumericValue),
    Volt(NumericValue),

    // Frequency
    Millihertz(NumericValue),
    Centihertz(NumericValue),
    Hertz(NumericValue),
    CyclePerMinute(NumericValue),

    // Jerk
    MeterPerSecondCubed(NumericValue),
    MeterPerMinutePerSecondSquared(NumericValue),

    // Length
    Millimeter(NumericValue),
    Centimeter(NumericValue),
    Meter(NumericValue),

    // LuminousIntensity
    Candela(NumericValue),

    // Mass
    Kilogram(NumericValue),

    // Pressure
    Pascal(NumericValue),
    Bar(NumericValue),

    // Ratio
    Ratio(NumericValue),

    // ThermodynamicTemperature
    Kelvin(NumericValue),
    DegreeCelsius(NumericValue),

    // Time
    Second(NumericValue),

    // Velocity
    MillimeterPerSecond(NumericValue),
    MeterPerSecond(NumericValue),
    MeterPerMinute(NumericValue),

    // VolumeRate
    CubicMeterPerSecond(NumericValue),
    LiterPerSecond(NumericValue),
    LiterPerMinute(NumericValue),
}

#[derive(Debug, Clone)]
pub enum ValueV2 {
    Enum(EnumValue),
    Boolean(BooleanValue),
    Integer(NumericValue),
    Float(NumericValue),
    Quantity {
        value: NumericValue,
        unit: Unit,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumValue {
    /// The set of allowed variants for this value. Required.
    pub variants: EnumVariants,
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
