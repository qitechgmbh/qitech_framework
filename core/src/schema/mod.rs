use core::fmt;
use std::fmt::Display;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use unic_langid::LanguageIdentifier;

use crate::ident::MachineIdentification;

pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

mod version;
pub use version::Version;

mod parser;
pub use parser::ParseError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineSchema {
    // --- meta data ---
    pub qms_version: Version,
    pub revision: u32,

    // --- interface ---
    pub name: String,
    pub identification: MachineIdentification,
    pub config_properties: StringMap<ScalarPropertyDefinition>,
    pub state_properties: StringMap<StatePropertyDefinition>,
    pub measurements: StringMap<MeasurementDefinition>,
    pub commands: StringMap<CommandDefinition>,
    pub events: StringMap<EventDefinition>,
}

impl MachineSchema {
    pub fn parse_str(s: &str) -> Result<Self, ParseError> {
        parser::parse_str(s)
    }
}

// --- scalar ---
pub type ConfigPropertyDefinition = ScalarPropertyDefinition;
pub type StatePropertyDefinition = ScalarPropertyDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarPropertyDefinition {
    pub kind: ScalarPropertyKind,
    pub nullable: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalarPropertyKind {
    Enum {
        /// Variants of the enum.
        variants: EnumVariants,
    },
    String,
    Boolean,
    Integer,
    Float {
        /// Representation of the float. E.g. plain, fraction, millimeter
        semantic: FloatSemantic,
    },
}

impl Display for ScalarPropertyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enum { variants } => {
                write!(f, "enum ")?;
                f.debug_list().entries(variants.list()).finish()
            }
            Self::String => write!(f, "string"),
            Self::Boolean => write!(f, "boolean"),
            Self::Integer => write!(f, "integer"),
            Self::Float { semantic } => write!(f, "{semantic}"),
        }
    }
}

// --- measurements ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementDefinition {
    pub kind: MeasurementKind,
    pub nullable: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementKind {
    Boolean,
    Integer {
        statistics: MeasurementStatistics,
    },
    Float {
        semantic: FloatSemantic,
        statistics: MeasurementStatistics,
    },
}

impl Display for MeasurementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "boolean"),
            Self::Integer { .. } => write!(f, "integer"),
            Self::Float { semantic, .. } => write!(f, "{semantic}"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementStatistics {
    /// Track the minimum observed value during each sampling cycle.
    /// Default is `false`.
    #[serde(default)]
    pub min: bool,

    /// Track the maximum observed value during each sampling cycle.
    /// Default is `false`.
    #[serde(default)]
    pub max: bool,

    /// Track the arithmetic mean of observed values during each sampling cycle.
    /// Default is `false`.
    #[serde(default)]
    pub avg: bool,

    /// Track the standard deviation of observed values during each sampling cycle.
    /// This provides a measure of variability or stability.
    /// Default is `false`.
    #[serde(default)]
    pub stddev: bool,
}

// --- command ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub metadata: Metadata,
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub fields: StringMap<EventField>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventField {
    pub kind: EventFieldKind,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventFieldKind {
    Object { fields: StringMap<EventField> },
    List { item: Box<EventField> },
    Enum { variants: EnumVariants },
    String,
    Boolean,
    Integer,
    Float { semantic: FloatSemantic },
}

// --- enum variants ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariants {
    values: StringMap<i64>,
    reverse: Map<i64, String>,
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

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn new() -> Self {
        Self {
            values: Default::default(),
            reverse: Default::default(),
        }
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

// --- metadata ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub description: LocalizedText,
}

impl Display for FloatSemantic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloatSemantic::Plain => write!(f, "!float"),
            FloatSemantic::Fraction => write!(f, "!fraction"),
            FloatSemantic::Percentage => write!(f, "!percentage"),
            FloatSemantic::Quantity(quantity) => write!(f, "!{quantity}"),
        }
    }
}

// --- float semantic ---
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FloatSemantic {
    Plain,
    Fraction,
    Percentage,
    Quantity(Quantity),
}

impl FromStr for FloatSemantic {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(v) = Quantity::from_str(s) {
            return Ok(Self::Quantity(v));
        }

        Ok(match s {
            "float" => Self::Plain,
            "fraction" => Self::Fraction,
            "percentage" => Self::Percentage,
            other => return Err(format!("invalid type {other}")),
        })
    }
}

impl FloatSemantic {
    pub fn as_str(&self) -> &'static str {
        match self {
            FloatSemantic::Plain => "float",
            FloatSemantic::Fraction => "fraction",
            FloatSemantic::Percentage => "percentage",
            FloatSemantic::Quantity(quantity) => quantity.as_str(),
        }
    }
}

// --- quantity ---
pub mod quantity {
    // includes mod generated { ... }
    include!(concat!(env!("OUT_DIR"), "/quantity.rs"));

    pub use generated::AccelerationUnit;
    pub use generated::AmountOfSubstanceUnit;
    pub use generated::AngleUnit;
    pub use generated::AngularAccelerationUnit;
    pub use generated::AngularJerkUnit;
    pub use generated::AngularVelocityUnit;
    pub use generated::ElectricCurrentUnit;
    pub use generated::ElectricPotentialUnit;
    pub use generated::FrequencyUnit;
    pub use generated::JerkUnit;
    pub use generated::LengthUnit;
    pub use generated::LuminousIntensityUnit;
    pub use generated::MassUnit;
    pub use generated::PressureUnit;
    pub use generated::Quantity;
    pub use generated::RatioUnit;
    pub use generated::ThermodynamicTemperatureUnit;
    pub use generated::TimeUnit;
    pub use generated::VelocityUnit;
    pub use generated::VolumeRateUnit;
}

pub use quantity::Quantity;
