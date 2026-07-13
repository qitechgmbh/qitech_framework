use serde::Deserialize;
use std::{fmt::Display, str::FromStr};
use super::{LocalizedText, EnumVariants, Range, StringMap, CommandParameter};

// TODO: rethink default definitions and arrays

#[derive(Debug, Clone)]
pub struct Command {
    pub description: LocalizedText,
    pub parameters: StringMap<CommandParameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterValue {
    Array(Box<ArrayValue>),
    Enum(EnumParameter),
    String(StringParameter),
    Boolean(BooleanParameter),
    Integer(NumericParameter<i64>),
    Float(NumericParameter<f64>),
    Fraction(NumericParameter<f64>),
    Percentage(NumericParameter<f64>),

    // Acceleration
    MeterPerSecondSquared(NumericParameter<f64>),
    MeterPerMinutePerSecond(NumericParameter<f64>),

    // AmountOfSubstance
    Mole(NumericParameter<f64>),

    // Angle
    Radian(NumericParameter<f64>),
    Degree(NumericParameter<f64>),
    Revolution(NumericParameter<f64>),

    // AngularAcceleration
    RadianPerSecondSquared(NumericParameter<f64>),
    DegreePerSecondSquared(NumericParameter<f64>),
    RevolutionPerMinutePerSecond(NumericParameter<f64>),

    // AngularJerk
    RadianPerSecondCubed(NumericParameter<f64>),
    DegreePerSecondCubed(NumericParameter<f64>),
    RevolutionPerMinutePerSecondSquared(NumericParameter<f64>),

    // AngularVelocity
    RadianPerSecond(NumericParameter<f64>),
    DegreePerSecond(NumericParameter<f64>),
    RevolutionPerSecond(NumericParameter<f64>),
    RevolutionPerMinute(NumericParameter<f64>),

    // ElectricCurrent
    Milliampere(NumericParameter<f64>),
    Centiampere(NumericParameter<f64>),
    Ampere(NumericParameter<f64>),

    // ElectricPotential
    Millivolt(NumericParameter<f64>),
    Centivolt(NumericParameter<f64>),
    Volt(NumericParameter<f64>),

    // Frequency
    Millihertz(NumericParameter<f64>),
    Centihertz(NumericParameter<f64>),
    Hertz(NumericParameter<f64>),
    CyclePerMinute(NumericParameter<f64>),

    // Jerk
    MeterPerSecondCubed(NumericParameter<f64>),
    MeterPerMinutePerSecondSquared(NumericParameter<f64>),

    // Length
    Millimeter(NumericParameter<f64>),
    Centimeter(NumericParameter<f64>),
    Meter(NumericParameter<f64>),

    // LuminousIntensity
    Candela(NumericParameter<f64>),

    // Mass
    Kilogram(NumericParameter<f64>),

    // Pressure
    Pascal(NumericParameter<f64>),
    Bar(NumericParameter<f64>),

    // Ratio
    Ratio(NumericParameter<f64>),

    // ThermodynamicTemperature
    Kelvin(NumericParameter<f64>),
    DegreeCelsius(NumericParameter<f64>),

    // Time
    Second(NumericParameter<f64>),

    // Velocity
    MillimeterPerSecond(NumericParameter<f64>),
    MeterPerSecond(NumericParameter<f64>),
    MeterPerMinute(NumericParameter<f64>),

    // VolumeRate
    CubicMeterPerSecond(NumericParameter<f64>),
    LiterPerSecond(NumericParameter<f64>),
    LiterPerMinute(NumericParameter<f64>),
}

// TODO: how to define default value ???
#[derive(Debug, Clone, Deserialize)]
pub struct ArrayValue {
    /// Type of each array element. Required.
    pub item: ParameterValue,
    /// item len range. Optional.
    /// Default is unbounded.
    pub range: Range<u32>,
}

#[derive(Debug, Clone)]
pub struct EnumParameter {
    /// Variants of the enum. Required.
    pub variants: EnumVariants,
    /// Default variant index. Optional.
    /// Default is `None`.
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StringParameter {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,

    /// Default value. Optional.
    /// Default is `None`.
    #[serde(default)]
    pub default: Option<String>,

    /// Allowed character length range. Optional.
    /// Default is unbounded.
    pub length: Range<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanParameter {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,

    /// Default value. Optional.
    /// Default is `None`.
    #[serde(default)]
    pub default: Option<bool>,
}

// TODO: pull into de_impl
// > default may not be null if not nullable
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericParameter<T>
where
    T: FromStr,
    T::Err: Display,
{
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,

    /// Default value. Optional.
    /// Default is `None`.
    #[serde(default)]
    pub default: Option<T>,

    /// Allowed range for this value. Optional.
    /// Default is unbounded.
    #[serde(default)]
    pub range: Range<T>,

    /// Increment between valid values. Optional.
    /// Default is `Unbounded`.
    #[serde(default)]
    pub step: Option<T>,
}
