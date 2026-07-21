use serde::{
    Deserialize, Deserializer,
    de::{
        self, EnumAccess, Error, MapAccess, SeqAccess, Visitor, value::{EnumAccessDeserializer, MapAccessDeserializer}
    },
};
use yaml_serde::Value;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};
use unic_langid::LanguageIdentifier;

use crate::schema::v1_0::{Unit, measurement, state};

use super::{
    raw, config, command,
    Command, EnumVariants, LocalizedText, Property, PropertyKind, Range, Schema, StringMap,
};

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = raw::MachineSchemaRaw::deserialize(deserializer)?;
        Schema::try_from(raw).map_err(D::Error::custom)
    }
}

impl<'de, V> Deserialize<'de> for Property<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let kind = PropertyKind::deserialize(deserializer)?;

        Ok(Self {
            description: LocalizedText::default(),
            kind,
        })
    }
}

impl<'de, V> Deserialize<'de> for PropertyKind<V>
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
            type Value = PropertyKind<V>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a property group or tagged value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let value = V::deserialize(EnumAccessDeserializer::new(data))?;

                Ok(PropertyKind::Value(value))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let group = StringMap::deserialize(MapAccessDeserializer::new(map))?;

                Ok(PropertyKind::Group(group))
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

impl<'de> Deserialize<'de> for EnumVariants {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnumVariantsVisitor;

        impl<'de> Visitor<'de> for EnumVariantsVisitor {
            type Value = EnumVariants;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of enum names or a map of enum name to integer")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut variants = EnumVariantsBuilder::new();
                let mut index = 0i64;

                while let Some(name) = seq.next_element::<String>()? {
                    variants.insert(name, index).map_err(Error::custom)?;
                    index += 1;
                }

                variants.finish().map_err(Error::custom)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut variants = EnumVariantsBuilder::new();

                while let Some((name, value)) = map.next_entry::<String, i64>()? {
                    variants.insert(name, value).map_err(Error::custom)?;
                }

                variants.finish().map_err(Error::custom)
            }
        }

        deserializer.deserialize_any(EnumVariantsVisitor)
    }
}

// --- config ---
impl<'de> Deserialize<'de> for config::Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let Value::Tagged(tagged) = value else {
            return Err(de::Error::custom("expected tagged value"));
        };

        let tag = &tagged.tag.to_string();
        match &tag.as_str()[1..] {
            "enum" => {
                let value = config::EnumValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Enum(value))
            }

            "string" => {
                let value = config::StringValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::String(value))
            }

            "boolean" => {
                let value = config::BooleanValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Boolean(value))
            }

            "integer" => {
                let value = config::IntegerValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Integer(value))
            }

            "float" => {
                let value = config::FloatValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Float(value))
            }

            "fraction" => {
                let value = config::FloatValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Fraction(value))
            }

            "percentage" => {
                let value = config::FloatValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Percentage(value))
            }

            tag => {
                let unit: Unit = yaml_serde::from_str(tag)
                    .map_err(de::Error::custom)?;

                let value = config::FloatValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;

                Ok(Self::Quantity { value, unit })
            }
        }
    }
}

impl<'de> Deserialize<'de> for config::EnumValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Helper {
            variants: EnumVariants,

            #[serde(default = "persistent_default")]
            pub persistent: bool,

            default: String,
        }

        let Helper {
            variants,
            persistent,
            default,
        } = Helper::deserialize(deserializer)?;

        if variants.get_int(&default).is_none() {
            return Err(D::Error::custom(format!("no such variant {:?}", default)));
        }

        Ok(Self {
            variants,
            persistent,
            default,
        })
    }
}

impl<'de> Deserialize<'de> for config::StringValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            #[serde(default)]
            nullable: bool,
            #[serde(default)]
            default: Option<String>,
            #[serde(default)]
            length: Range<u32>,
            #[serde(default = "persistent_default")]
            persistent: bool,
        }

        let Helper {
            nullable,
            default,
            length,
            persistent,
        } = Helper::deserialize(deserializer)?;

        if !nullable && default.is_none() {
            return Err(D::Error::custom(
                "`default` is required when `nullable` is `false`",
            ));
        }

        // ensure default itself lies in the provided range
        if let Some(v) = &default {
            let len = v.len() as u32;
            if !length.in_range(len) {
                return Err(D::Error::custom(format!(
                    "default value '{v}' (len = {len}) is outside the allowed length range {length}"
                )));
            }
        }

        Ok(Self {
            nullable,
            default,
            length,
            persistent,
        })
    }
}

impl<'de> Deserialize<'de> for config::BooleanValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Helper {
            #[serde(default)]
            pub nullable: bool,
            #[serde(default)]
            pub default: Option<bool>,
            #[serde(default = "persistent_default")]
            pub persistent: bool,
        }

        let Helper {
            nullable,
            default,
            persistent,
        } = Helper::deserialize(deserializer)?;

        if !nullable && default.is_none() {
            return Err(D::Error::custom(
                "`default` is required when `nullable` is `false`",
            ));
        }

        Ok(Self {
            nullable,
            default,
            persistent,
        })
    }
}

impl<'de, T> Deserialize<'de> for config::NumericValue<T>
where
    T: FromStr + Default + Deserialize<'de>,
    T::Err: Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Helper<T>
        where
            T: FromStr + Default,
            T::Err: Display,
        {
            #[serde(default)]
            pub nullable: bool,
            #[serde(default)]
            pub default: Option<T>,
            #[serde(default)]
            pub range: Range<T>,
            #[serde(default = "persistent_default")]
            pub persistent: bool,
        }

        let Helper {
            nullable,
            default,
            range,
            persistent,
        } = Helper::deserialize(deserializer)?;

        if !nullable && default.is_none() {
            return Err(D::Error::custom(
                "`default` is required when `nullable` is `false`",
            ));
        }

        Ok(Self {
            nullable,
            default,
            range,
            persistent,
        })
    }
}

// --- state ---
impl<'de> Deserialize<'de> for state::Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let Value::Tagged(tagged) = value else {
            return Err(de::Error::custom("expected tagged value"));
        };

        // strip '!' char
        let tag = &tagged.tag.to_string();
        match &tag.as_str()[1..] {
            "enum" => {
                let value = state::EnumValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Enum(value))
            }

            "string" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::String(value))
            }

            "boolean" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Boolean(value))
            }

            "integer" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Integer(value))
            }

            "float" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Float(value))
            }

            "fraction" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Fraction(value))
            }

            "percentage" => {
                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Percentage(value))
            }

            tag => {
                let unit: Unit = yaml_serde::from_str(tag)
                    .map_err(de::Error::custom)?;

                let value = state::ScalarValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;

                Ok(Self::Quantity { value, unit })
            }
        }
    }
}

// --- measurement ---
impl<'de> Deserialize<'de> for measurement::Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use measurement::{NumericValue, BooleanValue};

        let value = Value::deserialize(deserializer)?;

        let Value::Tagged(tagged) = value else {
            return Err(de::Error::custom("expected tagged value"));
        };

        let tag = &tagged.tag.to_string();
        match &tag.as_str()[1..] {
            "boolean" => {
                let value = BooleanValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Boolean(value))
            }

            "integer" => {
                let value = NumericValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Integer(value))
            }

            "float" => {
                let value = NumericValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Float(value))
            }

            "fraction" => {
                let value = NumericValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Fraction(value))
            }

            "percentage" => {
                let value = NumericValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;
                Ok(Self::Percentage(value))
            }

            tag => {
                let unit: Unit = yaml_serde::from_str(tag)
                    .map_err(de::Error::custom)?;

                let value = NumericValue::deserialize(tagged.value)
                    .map_err(de::Error::custom)?;

                Ok(Self::Quantity { value, unit })
            }
        }
    }
}

// --- command ---
impl<'de> Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let parameters = StringMap::deserialize(deserializer)?;

        Ok(Self {
            description: LocalizedText::default(),
            parameters,
        })
    }
}

impl<'de> Deserialize<'de> for command::EnumParameter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Helper {
            variants: EnumVariants,

            #[serde(default)]
            default: Option<String>,
        }

        let Helper { variants, default } = Helper::deserialize(deserializer)?;

        if let Some(d) = &default && variants.get_int(d).is_none() {
            return Err(D::Error::custom(format!("no such variant {:?}", d)));
        }

        Ok(Self { variants, default })
    }
}

// > utility types

impl<'de> Deserialize<'de> for raw::CommandDescriptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CommandDescriptionsVisitor;

        impl<'de> Visitor<'de> for CommandDescriptionsVisitor {
            type Value = raw::CommandDescriptions;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence of enum names or a map of enum name to integer")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut out = raw::CommandDescriptions {
                    description: Default::default(),
                    parameters: Default::default(),
                };

                // First phase: consume locale keys
                let first_child = loop {
                    let Some(key) = map.next_key::<String>()? else {
                        break None;
                    };
                    if !key.contains('-') {
                        break Some(key);
                    }
                    let lang = key
                        .parse::<LanguageIdentifier>()
                        .map_err(A::Error::custom)?;
                    let value = map.next_value::<String>()?;
                    out.description.insert(lang, value);
                };

                // Second phase: consume children
                if let Some(key) = first_child {
                    let value = map.next_value::<raw::DescriptionNode>()?;
                    out.parameters.insert(key, value);
                    while let Some(key) = map.next_key::<String>()? {
                        let value = map.next_value::<raw::DescriptionNode>()?;
                        out.parameters.insert(key, value);
                    }
                }

                Ok(out)
            }
        }

        deserializer.deserialize_any(CommandDescriptionsVisitor)
    }
}

// enum variants helper
#[derive(Debug, Clone)]
struct EnumVariantsBuilder {
    values: StringMap<i64>,
    reverse: HashMap<i64, String>,
}

impl EnumVariantsBuilder {
    fn new() -> Self {
        Self {
            values: Default::default(),
            reverse: Default::default(),
        }
    }

    fn insert(&mut self, name: String, value: i64) -> Result<(), String> {
        if self.values.contains_key(&name) {
            return Err(format!("duplicate enum key {:?}", name));
        }

        if self.reverse.contains_key(&value) {
            return Err(format!("duplicate enum value {:?}", value));
        }

        self.values.insert(name.clone(), value);
        self.reverse.insert(value, name);
        Ok(())
    }

    fn finish(self) -> Result<EnumVariants, &'static str> {
        if self.values.is_empty() {
            return Err("enum cannot be empty");
        }

        Ok(EnumVariants {
            values: self.values,
            reverse: self.reverse,
        })
    }
}

// misc
fn persistent_default() -> bool {
    true
}
