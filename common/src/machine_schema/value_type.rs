#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Array,
    Enum,
    String,
    Boolean,
    Integer,
    Float(FloatSemantic),
    Command,
    Event,
}

#[derive(Debug, Clone, Copy)]
pub enum FloatSemantic {
    Plain,
    Fraction,
    Percentage,
    Quantity(Quantity),
} 

// --- quantity ---
#[derive(Debug, Clone, Copy)]
pub enum Quantity {
    Acceleration(AccelerationUnit),
    AmountOfSubstance(AmountOfSubstanceUnit),
    Angle(AngleUnit),
    AngularAcceleration(AngularAccelerationUnit),
    AngularJerk(AngularJerkUnit),
    AngularVelocity(AngularVelocityUnit),
    ElectricCurrent(ElectricCurrentUnit),
    ElectricPotential(ElectricPotentialUnit),
    Frequency(FrequencyUnit),
    Jerk(JerkUnit),
    Length(LengthUnit),
    LuminousIntensity(LuminousIntensityUnit),
    Mass(MassUnit),
    Pressure(PressureUnit),
    Ratio(RatioUnit),
    ThermodynamicTemperature(ThermodynamicTemperatureUnit),
    Time(TimeUnit),
    Velocity(VelocityUnit),
    VolumeRate(VolumeRateUnit),
}

#[derive(Debug, Clone, Copy)]
pub enum AccelerationUnit {
    MeterPerSecondSquared,
    MeterPerMinutePerSecond,
}

#[derive(Debug, Clone, Copy)]
pub enum AmountOfSubstanceUnit {
    Mole,
}

#[derive(Debug, Clone, Copy)]
pub enum AngleUnit {
    Radian,
    Degree,
    Revolution,
}

#[derive(Debug, Clone, Copy)]
pub enum AngularAccelerationUnit {
    RadianPerSecondSquared,
    DegreePerSecondSquared,
    RevolutionPerMinutePerSecond,
}

#[derive(Debug, Clone, Copy)]
pub enum AngularJerkUnit {
    RadianPerSecondCubed,
    DegreePerSecondCubed,
    RevolutionPerMinutePerSecondSquared,
}

#[derive(Debug, Clone, Copy)]
pub enum AngularVelocityUnit {
    RadianPerSecond,
    DegreePerSecond,
    RevolutionPerSecond,
    RevolutionPerMinute,
}

#[derive(Debug, Clone, Copy)]
pub enum ElectricCurrentUnit {
    Milliampere,
    Centiampere,
    Ampere,
}

#[derive(Debug, Clone, Copy)]
pub enum ElectricPotentialUnit {
    Millivolt,
    Centivolt,
    Volt,
}

#[derive(Debug, Clone, Copy)]
pub enum FrequencyUnit {
    Millihertz,
    Centihertz,
    Hertz,
    CyclePerMinute,
}

#[derive(Debug, Clone, Copy)]
pub enum JerkUnit {
    MeterPerSecondCubed,
    MeterPerMinutePerSecondSquared,
}

#[derive(Debug, Clone, Copy)]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
}

#[derive(Debug, Clone, Copy)]
pub enum LuminousIntensityUnit {
    Candela,
}

#[derive(Debug, Clone, Copy)]
pub enum MassUnit {
    Kilogram,
}

#[derive(Debug, Clone, Copy)]
pub enum PressureUnit {
    Pascal,
    Bar,
}

#[derive(Debug, Clone, Copy)]
pub enum RatioUnit {
    Ratio,
}

#[derive(Debug, Clone, Copy)]
pub enum ThermodynamicTemperatureUnit {
    Kelvin,
    DegreeCelsius,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Second,
}

#[derive(Debug, Clone, Copy)]
pub enum VelocityUnit {
    MillimeterPerSecond,
    MeterPerSecond,
    MeterPerMinute,
}

#[derive(Debug, Clone, Copy)]
pub enum VolumeRateUnit {
    CubicMeterPerSecond,
    LiterPerSecond,
    LiterPerMinute,
} 

// --- deserialize implemenations ---
include!(concat!(env!("OUT_DIR"), "/parse_value_type.rs"));