use std::{collections::HashMap, fmt::{self, Display, Formatter}};
use indexmap::IndexMap;
use unic_langid::LanguageIdentifier;
use crate::{
    MachineIdentification,
    schema::{AnyMachineSchema, QmsVersion}
};

pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

pub type ConfigProperty = Property<config::ValueV2>;
pub type StateProperty = Property<state::ValueV2>;
pub type MeasurementProperty = Property<measurement::ValueV2>;
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
    let schema = yaml_serde::from_str::<MachineSchema>(data)?;
    Ok(AnyMachineSchema::V1_0(schema))
}

#[derive(Debug, Clone)]
pub struct MachineSchema {
    pub qms_version: QmsVersion,
    pub name: String,
    pub schema_revision: u32,
    pub identification: MachineIdentification,
    pub config: StringMap<ConfigProperty>,
    pub state: StringMap<StateProperty>,
    pub measurements: StringMap<MeasurementProperty>,
    pub commands: StringMap<Command>,
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

    pub fn get(&self, name: &str) -> Option<i64> {
        self.values.get(name).copied()
    }

    pub fn get_name(&self, value: i64) -> Option<&str> {
        self.reverse.get(&value).map(String::as_str)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, String, i64> {
        self.values.iter()
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

// --- physical units --- 
#[derive(Debug, Clone)]
pub enum Unit {
    MeterPerSecondSquared,
    MeterPerMinutePerSecond,
    Mole,
    Radian,
    Degree,
    Revolution,
    RadianPerSecondSquared,
    DegreePerSecondSquared,
    RevolutionPerMinutePerSecond,
    RadianPerSecondCubed,
    DegreePerSecondCubed,
    RevolutionPerMinutePerSecondSquared,
    RadianPerSecond,
    DegreePerSecond,
    RevolutionPerSecond,
    RevolutionPerMinute,
    Milliampere,
    Centiampere,
    Ampere,
    Millivolt,
    Centivolt,
    Volt,
    Millihertz,
    Centihertz,
    Hertz,
    CyclePerMinute,
    MeterPerSecondCubed,
    MeterPerMinutePerSecondSquared,
    Millimeter,
    Centimeter,
    Meter,
    Candela,
    Kilogram,
    Pascal,
    Bar,
    Ratio,
    Kelvin,
    DegreeCelsius,
    Second,
    MillimeterPerSecond,
    MeterPerSecond,
    MeterPerMinute,
    CubicMeterPerSecond,
    LiterPerSecond,
    LiterPerMinute,
}
