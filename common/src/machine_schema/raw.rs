use serde::Deserialize;

use crate::{MachineIdentification, Version};
use super::{
    LocalizedText, Node, NodeKind,
    MachineSchema, StringMap,
    config_property, state_property, measurement
};

#[derive(Debug, Clone, Deserialize)]
/// Raw representation of the yaml doc
pub struct MachineSchemaRaw {
    // --- meta data ---
    pub qf_version: Version,
    pub revision: u32,

    // --- interface ---
    pub name: String,
    pub identification: MachineIdentification,

    #[serde(default)]
    pub config: StringMap<Node<config_property::Value>>,

    #[serde(default)]
    pub state: StringMap<Node<state_property::Value>>,

    #[serde(default)]
    pub measurements: StringMap<Node<measurement::Value>>,

    // #[serde(default)]
    // pub commands: StringMap<Command>,

    #[serde(default)]
    pub descriptions: Descriptions,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Descriptions {
    #[serde(default)]
    pub config: StringMap<DescriptionNode>,

    #[serde(default)]
    pub state: StringMap<DescriptionNode>,

    #[serde(default)]
    pub measurements: StringMap<DescriptionNode>,

    // #[serde(default)]
    // pub commands: StringMap<CommandDescriptions>,
}

#[derive(Debug, Clone)]
pub struct CommandDescriptions {
    pub description: LocalizedText,
    pub parameters: StringMap<DescriptionNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DescriptionNode {
    Branch(StringMap<DescriptionNode>),
    Leaf(LocalizedText),
}

impl TryFrom<MachineSchemaRaw> for MachineSchema {
    type Error = String;

    #[rustfmt::skip]
    fn try_from(mut raw: MachineSchemaRaw) -> Result<Self, Self::Error> {
        let config = merge_with_properties(
            "config", "config", 
            raw.descriptions.config, 
            raw.config,
        )?;

        let state = merge_with_properties(
            "state", "state", 
            raw.descriptions.state, 
            raw.state,
        )?;

        let measurements = merge_with_properties(
            "measurements", "measurements",
            raw.descriptions.measurements,
            raw.measurements,
        )?;

        // let mut commands = StringMap::new();
        // for (name, descriptions) in raw.descriptions.commands {
        //     let Some(mut command) = raw.commands.shift_remove(&name) else {
        //         return Err(format!("description for unknown property: commands.{name}"));
        //     };
// 
        //     command.description = descriptions.description;
        //     command.parameters = merge_with_properties(
        //         "commands", "commands",
        //         descriptions.parameters,
        //         command.parameters,
        //     )?;
// 
        //     commands.insert(name, command);
        // }
        
        Ok(Self {
            qf_version: raw.qf_version,
            name: raw.name,
            revision: raw.revision,
            identification: raw.identification,
            config_properties: config,
            state_properties: state,
            measurements,
            commands: raw.commands,
        })
    }
}

pub fn merge_with_properties<V>(
    section: &str,
    key: &str,
    descs: StringMap<DescriptionNode>,
    mut props: StringMap<Node<V>>,
) -> Result<StringMap<Node<V>>, String> {
    for (name, node) in descs {
        let Some(prop) = props.get_mut(&name) else {
            return Err(format!("description for unknown {section} property: {key}.{name}"));
        };

        match (&mut prop.kind, node) {
            (NodeKind::Leaf(_), DescriptionNode::Leaf(desc)) => {
                prop.description = desc;
            }
            (NodeKind::Branch(children), DescriptionNode::Branch(child_descs)) => {
                let nested_key = format!("{key}.{name}");
                let merged = merge_with_properties(
                    section,
                    &nested_key,
                    child_descs,
                    std::mem::take(children),
                )?;
                *children = merged;
            }
            (NodeKind::Leaf(_), DescriptionNode::Branch(_)) => {
                return Err(format!(
                    "expected a description for {section} property {key}.{name}, found a group"
                ));
            }
            (NodeKind::Branch(_), DescriptionNode::Leaf(_)) => {
                return Err(format!(
                    "expected a group of descriptions for {section} property {key}.{name}, found a single description"
                ));
            }
        }
    }

    Ok(props)
}
