use serde::Deserialize;

use crate::{MachineIdentification, Version};
use super::{
    MachineSchema, StringMap, Node, NodeKind, NodeMetadata, Command, Event,
    config_property, state_property, measurement
};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identification {
    pub name: String,
    pub vendor_id: u16,
    pub machine_id: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw representation of the yaml doc
pub struct MachineSchemaRaw {
    // --- meta data ---
    pub qms_version: Version,
    pub revision: u32,

    // --- interface ---
    pub identification: Identification,

    #[serde(default)]
    pub config: StringMap<Node<config_property::ConfigPropertyValue>>,

    #[serde(default)]
    pub state: StringMap<Node<state_property::StatePropertyValue>>,

    #[serde(default)]
    pub measurements: StringMap<Node<measurement::MeasurementValue>>,

    #[serde(default)]
    pub commands: StringMap<Node<Command>>,

    #[serde(default)]
    pub events: StringMap<Node<Event>>,

    #[serde(default)]
    pub metadata: ResourcesMetadata,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcesMetadata {
    #[serde(default)]
    pub config: StringMap<MetadataNode>,

    #[serde(default)]
    pub state: StringMap<MetadataNode>,

    #[serde(default)]
    pub measurements: StringMap<MetadataNode>,

    #[serde(default)]
    pub commands: StringMap<MetadataNode>,

    #[serde(default)]
    pub events: StringMap<MetadataNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetadataNode {
    Branch(StringMap<MetadataNode>),
    Leaf(NodeMetadata),
}

impl TryFrom<MachineSchemaRaw> for MachineSchema {
    type Error = String;

    #[rustfmt::skip]
    fn try_from(raw: MachineSchemaRaw) -> Result<Self, Self::Error> {
        let config = merge_with_metadata(
            "config", "config", 
            raw.metadata.config, 
            raw.config,
        )?;

        let state = merge_with_metadata(
            "state", "state", 
            raw.metadata.state, 
            raw.state,
        )?;

        let measurements = merge_with_metadata(
            "measurements", "measurements",
            raw.metadata.measurements,
            raw.measurements,
        )?;

        let commands = merge_with_metadata(
            "commands", "commands",
            raw.metadata.commands,
            raw.commands,
        )?;
        
        let events = merge_with_metadata(
            "events", "events",
            raw.metadata.events,
            raw.events,
        )?;

        let Identification { name, vendor_id, machine_id } = raw.identification;

        Ok(Self {
            qms_version: raw.qms_version,
            name,
            revision: raw.revision,
            identification: MachineIdentification { 
                vendor_id, 
                machine_id, 
            },
            config_properties: config,
            state_properties: state,
            measurements,
            commands,
            events,
        })
    }
}

fn merge_with_metadata<V>(
    section: &str,
    key: &str,
    metadata: StringMap<MetadataNode>,
    mut props: StringMap<Node<V>>,
) -> Result<StringMap<Node<V>>, String> {
    for (name, node) in metadata {
        let Some(prop) = props.get_mut(&name) else {
            return Err(format!("description for unknown {section} property: {key}.{name}"));
        };

        match (&mut prop.kind, node) {
            (NodeKind::Leaf(_), MetadataNode::Leaf(metadata)) => {
                prop.metadata = metadata;
            }
            (NodeKind::Branch(children), MetadataNode::Branch(child_descs)) => {
                let nested_key = format!("{key}.{name}");
                let merged = merge_with_metadata(
                    section,
                    &nested_key,
                    child_descs,
                    std::mem::take(children),
                )?;
                *children = merged;
            }
            (NodeKind::Leaf(_), MetadataNode::Branch(_)) => {
                return Err(format!(
                    "expected a description for {section} property {key}.{name}, found a group"
                ));
            }
            (NodeKind::Branch(_), MetadataNode::Leaf(_)) => {
                return Err(format!(
                    "expected a group of descriptions for {section} property {key}.{name}, found a single description"
                ));
            }
        }
    }

    Ok(props)
}
