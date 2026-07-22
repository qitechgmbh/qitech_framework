use std::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use unic_langid::LanguageIdentifier;

pub mod quantity {
    // generated module needs this
    use super::*;

    // includes mod generated { ... }
    include!(concat!(env!("OUT_DIR"), "/quantity.rs"));

    pub use generated::Quantity;
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
    pub use generated::RatioUnit;
    pub use generated::ThermodynamicTemperatureUnit;
    pub use generated::TimeUnit;
    pub use generated::VelocityUnit;
    pub use generated::VolumeRateUnit;
}

pub use quantity::Quantity;
pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Object,
    Array,
    Enum,
    String,
    Boolean,
    Integer,
    Float(FloatSemantic),
    Command,
    Event,
}

impl Type {
    pub fn parse(tag: &str) -> Result<Self, String> {
        if let Some(v) = Quantity::parse(tag) {
            return Ok(Self::Float(FloatSemantic::Quantity(v)));
        }

        Ok(match tag {
            "command" => Self::Command,
            "event" => Self::Event,
            "object" => Self::Object,
            "array" => Self::Array,
            "enum" => Self::Enum,
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "float" => Self::Float(FloatSemantic::Plain),
            "fraction" => Self::Float(FloatSemantic::Fraction),
            "percentage" => Self::Float(FloatSemantic::Percentage),
            other => return Err(format!("invalid type {other}")),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FloatSemantic {
    Plain,
    Fraction,
    Percentage,
    Quantity(Quantity),
}

#[derive(Debug, Clone)]
pub struct Node<V> {
    pub kind: NodeKind<V>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone)]
pub enum NodeKind<V> {
    Branch(StringMap<Node<V>>),
    Leaf(V),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub description: LocalizedText,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub description: LocalizedText,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Range<T> {
    #[default]
    Unbounded,
    Min(T),
    Max(T),
    Between { min: T, max: T },
}

impl<T> Range<T> 
where 
    T: Copy + PartialOrd
{
    pub fn in_range(self, value: T) -> bool {
        match self {
            Range::Unbounded => true,
            Range::Min(min) => value >= min,
            Range::Max(max) => value <= max,
            Range::Between { min, max } => min <= value && value <= max,
        }
    }
}

// --- display implementations ---
impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Type::Object => write!(f, "!object"),
            Type::Array => write!(f, "!array"),
            Type::Enum => write!(f, "!enum"),
            Type::String => write!(f, "!string"),
            Type::Boolean => write!(f, "!boolean"),
            Type::Integer => write!(f, "!integer"),
            Type::Float(semantic) => write!(f, "{semantic}"),
            Type::Command => write!(f, "!command"),
            Type::Event => write!(f, "!event"),
        }
    }
}

impl Display for FloatSemantic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FloatSemantic::Plain => write!(f, "!float"),
            FloatSemantic::Fraction => write!(f, "!fraction"),
            FloatSemantic::Percentage => write!(f, "!percentage"),
            FloatSemantic::Quantity(quantity) => write!(f, "!{quantity}"),
        }
    }
}

impl<T: Display> Display for Range<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Range::Unbounded => write!(f, ".."),
            Range::Min(min) => write!(f, "{}..", min),
            Range::Max(max) => write!(f, "..{}", max),
            Range::Between { min, max } => write!(f, "{}..{}", min, max),
        }
    }
}

// --- deserialize implemenations ---
use std::str::FromStr;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use serde::de::{Error, Deserializer, Visitor, EnumAccess, MapAccess};
use serde::de::value::{EnumAccessDeserializer, MapAccessDeserializer};

impl<'de, V> Deserialize<'de> for Node<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            kind: NodeKind::deserialize(deserializer)?,
            metadata: NodeMetadata::default(),
        })
    }
}

impl<'de, V> Deserialize<'de> for NodeKind<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor<V>(PhantomData<V>);

        impl<'de, V> Visitor<'de> for KindVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = NodeKind<V>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a property group or tagged value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let leaf = V::deserialize(EnumAccessDeserializer::new(data))?;
                Ok(NodeKind::Leaf(leaf))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let branch = StringMap::deserialize(MapAccessDeserializer::new(map))?;
                Ok(NodeKind::Branch(branch))
            }
        }

        deserializer.deserialize_any(KindVisitor(PhantomData))
    }
}

impl<'de, T> Deserialize<'de> for Range<T>
where
    T: FromStr,
    T::Err: Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        if value == ".." {
            return Ok(Range::Unbounded);
        }

        if let Some(min) = value.strip_suffix("..") {
            let min = min.parse().map_err(D::Error::custom)?;
            return Ok(Range::Min(min));
        }

        if let Some(max) = value.strip_prefix("..") {
            let max = max.parse().map_err(D::Error::custom)?;
            return Ok(Range::Max(max));
        }

        if let Some((min, max)) = value.split_once("..") {
            let min = min.parse().map_err(D::Error::custom)?;
            let max = max.parse().map_err(D::Error::custom)?;
            return Ok(Range::Between { min, max });
        }

        Err(D::Error::custom(format!("invalid range format: {}", value)))
    }
}
