use core::fmt;
use std::fmt::Display;

use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use unic_langid::LanguageIdentifier;


pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

/// Reference to a unit by its identifier, e.g. `si.millimeter`.
/// Placeholder until the unit model is reworked.
pub type UnitRef = String;

mod version;
pub use version::Version;

mod parser;
pub use parser::ParseError;

mod enum_variants;
pub use enum_variants::EnumVariants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineSchema {
    // --- metadata ---
    pub qms_version: Version,
    pub revision: u32,

    // --- interface ---
    pub name: String,
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
    pub ty: ScalarType,
    pub optional: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalarType {
    Enum {
        /// Variants of the enum.
        variants: EnumVariants,
    },
    String,
    Boolean,
    Numeric {
        ty: NumericType,
        unit: Option<UnitRef>,
    },
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enum { variants } => {
                write!(f, "enum(")?;
                for (i, (name, _)) in variants.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name}")?;
                }
                write!(f, ")")
            }
            Self::String { .. } => write!(f, "string"),
            Self::Boolean => write!(f, "boolean"),
            Self::Numeric { ty, unit } => fmt_numeric(f, ty, unit),
        }
    }
}

// --- numeric ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericType {
    /// Signed 64-bit integer.
    Integer,
    /// Unsigned 64-bit integer.
    IntegerUnsigned,
    /// 64-bit floating point.
    Float,
}

impl NumericType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::IntegerUnsigned => "integer_unsigned",
            Self::Float => "float",
        }
    }
}

impl Display for NumericType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn fmt_numeric(
    f: &mut fmt::Formatter<'_>,
    ty: &NumericType,
    unit: &Option<UnitRef>,
) -> fmt::Result {
    match unit {
        Some(unit) => write!(f, "{ty}({unit})"),
        None => write!(f, "{ty}"),
    }
}

// --- measurements ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementDefinition {
    pub ty: MeasurementType,
    pub nullable: bool,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementType {
    Boolean,
    Numeric {
        ty: NumericType,
        unit: Option<UnitRef>,
        statistics: MeasurementStatistics,
    },
}

impl Display for MeasurementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "boolean"),
            Self::Numeric { ty, unit, .. } => fmt_numeric(f, ty, unit),
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
    Object {
        fields: StringMap<EventField>,
    },
    List {
        item: Box<EventField>,
    },
    Enum {
        variants: EnumVariants,
    },
    String,
    Boolean,
    Numeric {
        ty: NumericType,
        unit: Option<UnitRef>,
    },
}

// --- metadata ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub description: LocalizedText,
}
