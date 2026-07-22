use std::{collections::HashMap, fmt::{self, Display, Formatter}};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;
use crate::{
    MachineIdentification,
    schema::{AnyMachineSchema, QmsVersion}
};

pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

pub type ConfigProperty = Property<config::Value>;
pub type StateProperty = Property<state::Value>;
pub type MeasurementProperty = Property<measurement::Value>;
pub type CommandParameter = Property<command::ParameterValue>;
pub use command::Command;

pub mod config;
pub mod state;
pub mod measurement;
pub mod command;
mod de_impl;
mod raw;

pub const VERSION: QmsVersion = QmsVersion { major: 1, minor: 0 };

pub(crate) fn parse(data: &str) -> yaml_serde::Result<AnyMachineSchema> {
    let schema = yaml_serde::from_str::<Schema>(data)?;
    Ok(AnyMachineSchema::V1_0(schema))
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub qms_version: QmsVersion,
    pub name: String,
    pub schema_revision: u32,
    pub identification: MachineIdentification,
    pub config_properties: StringMap<ConfigProperty>,
    pub state_properties: StringMap<StateProperty>,
    pub measurements: StringMap<MeasurementProperty>,
    pub commands: StringMap<Command>,
    // events
}

impl Schema {
    pub fn find_config_property<'a>(&'a self, name: &str) -> Option<&'a config::Value> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.config_properties.get(first)?;
        Self::walk_property(property, parts)
    }

    pub fn find_state_property<'a>(&'a self, name: &str) -> Option<&'a state::Value> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.state_properties.get(first)?;
        Self::walk_property(property, parts)
    }

    fn walk_property<'a, 'b, T, I>(
        property: &'a Property<T>,
        mut parts: I,
    ) -> Option<&'a T>
    where
        I: Iterator<Item = &'b str>,
    {
        match &property.kind {
            PropertyKind::Value(value) => {
                // leaf: only valid if path is exhausted
                if parts.next().is_none() {
                    Some(value)
                } else {
                    None
                }
            }

            PropertyKind::Group(children) => {
                let next = parts.next()?;
                let child = children.get(next)?;
                Self::walk_property(child, parts)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Property<V> {
    pub description: LocalizedText,
    pub kind: PropertyKind<V>,
}

#[derive(Debug, Clone)]
pub enum PropertyKind<V> {
    Group(StringMap<Property<V>>),
    Value(V),
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Range<T> {
    #[default]
    Unbounded,
    Min(T),
    Max(T),
    Between { min: T, max: T },
}

impl<T: PartialOrd> Range<T> {
    pub fn in_range(self, value: T) -> bool {
        match self {
            Range::Unbounded => true,
            Range::Min(min) => value >= min,
            Range::Max(max) => value <= max,
            Range::Between { min, max } => min <= value && value <= max,
        }
    }
}

impl<T: Display> Display for Range<T> 
where 
    T: Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Range::Unbounded => write!(f, ".."),
            Range::Min(min) => write!(f, "{}..", min),
            Range::Max(max) => write!(f, "..{}", max),
            Range::Between { min, max } => write!(f, "{}..{}", min, max),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnumVariants {
    values: StringMap<i64>,
    reverse: HashMap<i64, String>,
}

impl EnumVariants {
    pub fn first(&self) -> (&String, &i64) {
        self.values.first().expect("Cannot be empty")
    }

    pub fn last(&self) -> (&String, &i64) {
        self.values.last().expect("Cannot be empty")
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.values.get(name).copied()
    }

    pub fn get_name(&self, value: i64) -> Option<&str> {
        self.reverse.get(&value).map(String::as_str)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, i64> {
        self.values.iter()
    }

    pub fn list(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a> IntoIterator for &'a EnumVariants {
    type Item = (&'a String, &'a i64);
    type IntoIter = indexmap::map::Iter<'a, String, i64>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

impl IntoIterator for EnumVariants {
    type Item = (String, i64);
    type IntoIter = indexmap::map::IntoIter<String, i64>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatSemantic {
    Plain,
    Fraction,
    Percentage,
    Quantity(Quantity),
}

// --- quantity ---

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccelerationUnit {
    MeterPerSecondSquared,
    MeterPerMinutePerSecond,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmountOfSubstanceUnit {
    Mole,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngleUnit {
    Radian,
    Degree,
    Revolution,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngularAccelerationUnit {
    RadianPerSecondSquared,
    DegreePerSecondSquared,
    RevolutionPerMinutePerSecond,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngularJerkUnit {
    RadianPerSecondCubed,
    DegreePerSecondCubed,
    RevolutionPerMinutePerSecondSquared,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AngularVelocityUnit {
    RadianPerSecond,
    DegreePerSecond,
    RevolutionPerSecond,
    RevolutionPerMinute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectricCurrentUnit {
    Milliampere,
    Centiampere,
    Ampere,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectricPotentialUnit {
    Millivolt,
    Centivolt,
    Volt,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrequencyUnit {
    Millihertz,
    Centihertz,
    Hertz,
    CyclePerMinute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JerkUnit {
    MeterPerSecondCubed,
    MeterPerMinutePerSecondSquared,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LuminousIntensityUnit {
    Candela,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MassUnit {
    Kilogram,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureUnit {
    Pascal,
    Bar,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatioUnit {
    Ratio,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermodynamicTemperatureUnit {
    Kelvin,
    DegreeCelsius,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Second,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VelocityUnit {
    MillimeterPerSecond,
    MeterPerSecond,
    MeterPerMinute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolumeRateUnit {
    CubicMeterPerSecond,
    LiterPerSecond,
    LiterPerMinute,
}