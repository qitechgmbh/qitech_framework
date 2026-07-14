use serde::Deserialize ;
use super::{EnumVariants, Unit};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Enum(EnumValue),
    String(ScalarValue),
    Boolean(ScalarValue),
    Integer(ScalarValue),
    Float(ScalarValue),
    Fraction(ScalarValue),
    Percentage(ScalarValue),

    // Acceleration
    MeterPerSecondSquared(ScalarValue),
    MeterPerMinutePerSecond(ScalarValue),

    // AmountOfSubstance
    Mole(ScalarValue),

    // Angle
    Radian(ScalarValue),
    Degree(ScalarValue),
    Revolution(ScalarValue),

    // AngularAcceleration
    RadianPerSecondSquared(ScalarValue),
    DegreePerSecondSquared(ScalarValue),
    RevolutionPerMinutePerSecond(ScalarValue),

    // AngularJerk
    RadianPerSecondCubed(ScalarValue),
    DegreePerSecondCubed(ScalarValue),
    RevolutionPerMinutePerSecondSquared(ScalarValue),

    // AngularVelocity
    RadianPerSecond(ScalarValue),
    DegreePerSecond(ScalarValue),
    RevolutionPerSecond(ScalarValue),
    RevolutionPerMinute(ScalarValue),

    // ElectricCurrent
    Milliampere(ScalarValue),
    Centiampere(ScalarValue),
    Ampere(ScalarValue),

    // ElectricPotential
    Millivolt(ScalarValue),
    Centivolt(ScalarValue),
    Volt(ScalarValue),

    // Frequency
    Millihertz(ScalarValue),
    Centihertz(ScalarValue),
    Hertz(ScalarValue),
    CyclePerMinute(ScalarValue),

    // Jerk
    MeterPerSecondCubed(ScalarValue),
    MeterPerMinutePerSecondSquared(ScalarValue),

    // Length
    Millimeter(ScalarValue),
    Centimeter(ScalarValue),
    Meter(ScalarValue),

    // LuminousIntensity
    Candela(ScalarValue),

    // Mass
    Kilogram(ScalarValue),

    // Pressure
    Pascal(ScalarValue),
    Bar(ScalarValue),

    // Ratio
    Ratio(ScalarValue),

    // ThermodynamicTemperature
    Kelvin(ScalarValue),
    DegreeCelsius(ScalarValue),

    // Time
    Second(ScalarValue),

    // Velocity
    MillimeterPerSecond(ScalarValue),
    MeterPerSecond(ScalarValue),
    MeterPerMinute(ScalarValue),

    // VolumeRate
    CubicMeterPerSecond(ScalarValue),
    LiterPerSecond(ScalarValue),
    LiterPerMinute(ScalarValue),
}

#[derive(Debug, Clone)]
pub enum ValueV2 {
    Enum(EnumValue),
    Boolean(ScalarValue),
    Integer(ScalarValue),
    Float(ScalarValue),
    Quantity {
        value: ScalarValue,
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
pub struct ScalarValue {
    /// Whether this value is allowed to be null. Optional.
    /// Default is `false`.
    #[serde(default)]
    pub nullable: bool,
}
