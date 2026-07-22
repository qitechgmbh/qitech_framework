use std::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use unic_langid::LanguageIdentifier;

pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

#[derive(Debug, Clone)]
pub struct Node<V> {
    pub description: LocalizedText,
    pub kind: NodeKind<V>,
}

#[derive(Debug, Clone)]
pub enum NodeKind<V> {
    Branch(StringMap<Node<V>>),
    Leaf(V),
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

// --- deserialize implemenations ---
use std::str::FromStr;
use std::marker::PhantomData;
use serde::Deserialize;
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
        let kind = NodeKind::deserialize(deserializer)?;

        Ok(Self {
            description: LocalizedText::default(),
            kind,
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
                let value = V::deserialize(EnumAccessDeserializer::new(data))?;

                Ok(NodeKind::Leaf(value))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let group = StringMap::deserialize(MapAccessDeserializer::new(map))?;

                Ok(NodeKind::Branch(group))
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
