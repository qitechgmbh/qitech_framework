use crate::{MachineIdentification, Version};

pub type ConfigProperty = Node<config_property::Value>;
pub type StateProperty = Node<state_property::Value>;
pub type MeasurementProperty = Node<measurement::Value>;

mod types;
use types::Map;
use types::StringMap;
pub use types::LocalizedText;
pub use types::Node;
pub use types::NodeKind;
pub use types::Range;

mod enum_variants;
pub use enum_variants::EnumVariants;

mod value_type;
pub use value_type::ValueType;
pub use value_type::FloatSemantic;

pub mod config_property;
pub mod state_property;
pub mod measurement;
// pub mod command;
mod raw;

#[derive(Debug, Clone)]
pub struct MachineSchema {
    // --- meta data ---
    pub qf_version: Version,
    pub revision: u32,

    // --- interface ---
    pub name: String,
    pub identification: MachineIdentification,
    pub config_properties: StringMap<ConfigProperty>,
    pub state_properties: StringMap<StateProperty>,
    pub measurements: StringMap<MeasurementProperty>,
    // pub commands: StringMap<Node<Command>>,
    // events
}

impl MachineSchema {
    pub fn find_config_property<'a>(&'a self, name: &str) -> Option<&'a config_property::ValueKind> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.config_properties.get(first)?;
        Self::walk_node(property, parts)
    }

    pub fn find_state_property<'a>(&'a self, name: &str) -> Option<&'a state_property::Value> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.state_properties.get(first)?;
        Self::walk_node(property, parts)
    }

    fn walk_node<'a, 'b, T, I>(
        property: &'a Node<T>,
        mut parts: I,
    ) -> Option<&'a T>
    where
        I: Iterator<Item = &'b str>,
    {
        match &property.kind {
            NodeKind::Leaf(value) => {
                // leaf: only valid if path is exhausted
                if parts.next().is_none() {
                    Some(value)
                } else {
                    None
                }
            }

            NodeKind::Branch(children) => {
                let next = parts.next()?;
                let child = children.get(next)?;
                Self::walk_node(child, parts)
            }
        }
    }
}

use serde::de::{Deserializer, Deserialize, Error};

impl<'de> Deserialize<'de> for MachineSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = raw::MachineSchemaRaw::deserialize(deserializer)?;
        MachineSchema::try_from(raw).map_err(D::Error::custom)
    }
}
