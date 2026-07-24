use crate::MachineIdentification;

mod version;
pub use version::Version;

mod types;
pub use types::FieldMetadata;
pub use types::FloatSemantic;
pub use types::LocalizedText;
use types::Map;
pub use types::Node;
pub use types::NodeKind;
pub use types::NodeMetadata;
pub use types::Range;
use types::StringMap;
pub use types::Type;
pub use types::quantity::Quantity;
pub use types::quantity::{self};

mod enum_variants;
pub use enum_variants::EnumVariants;

mod config_property;
pub use config_property::ConfigPropertyValue;
pub use config_property::ConfigPropertyValueKind;

mod state_property;
pub use state_property::StatePropertyValue;
pub use state_property::StatePropertyValueKind;

mod measurement;
pub use measurement::MeasurementStatistics;
pub use measurement::MeasurementValue;
pub use measurement::MeasurementValueKind;

mod command;
pub use command::Command;
pub use command::CommandField;
pub use command::CommandFieldKind;

mod event;
pub use event::Event;
pub use event::EventField;
pub use event::EventFieldKind;

mod raw;

#[derive(Debug, Clone)]
pub struct MachineSchema {
    // --- meta data ---
    pub qms_version: Version,
    pub revision: u32,

    // --- interface ---
    pub name: String,
    pub identification: MachineIdentification,
    pub config_properties: StringMap<Node<ConfigPropertyValue>>,
    pub state_properties: StringMap<Node<state_property::StatePropertyValue>>,
    pub measurements: StringMap<Node<measurement::MeasurementValue>>,
    pub commands: StringMap<Node<Command>>,
    pub events: StringMap<Node<Event>>,
}

impl MachineSchema {
    pub fn from_yaml_str(value: &str) -> yaml_serde::Result<Self> {
        yaml_serde::from_str(value)
    }
}

impl MachineSchema {
    pub fn find_config_property<'a>(
        &'a self,
        name: &str,
    ) -> Option<&'a config_property::ConfigPropertyValue> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.config_properties.get(first)?;
        Self::walk_node(property, parts)
    }

    pub fn find_state_property<'a>(
        &'a self,
        name: &str,
    ) -> Option<&'a state_property::StatePropertyValue> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let property = self.state_properties.get(first)?;
        Self::walk_node(property, parts)
    }

    pub fn find_measurement<'a>(&'a self, name: &str) -> Option<&'a MeasurementValue> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let measurement = self.measurements.get(first)?;
        Self::walk_node(measurement, parts)
    }

    pub fn find_command<'a>(&'a self, name: &str) -> Option<&'a Command> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let command = self.commands.get(first)?;
        Self::walk_node(command, parts)
    }

    pub fn find_event<'a>(&'a self, name: &str) -> Option<&'a Event> {
        let mut parts = name.split('.');
        let first = parts.next()?;
        let event = self.events.get(first)?;
        Self::walk_node(event, parts)
    }

    fn walk_node<'a, 'b, T, I>(property: &'a Node<T>, mut parts: I) -> Option<&'a T>
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

// --- deserialize implemenations ---
use serde::de::Deserialize;
use serde::de::Deserializer;
use serde::de::Error;

impl<'de> Deserialize<'de> for MachineSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = raw::MachineSchemaRaw::deserialize(deserializer)?;

        if !Version::is_supported(raw.qms_version) {
            return Err(D::Error::custom(format!(
                "Unsupported version: {}",
                raw.qms_version
            )));
        }

        MachineSchema::try_from(raw).map_err(D::Error::custom)
    }
}
