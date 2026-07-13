use serde::Deserialize;

use crate::MachineIdentification;
use super::{
    LocalizedText, Property, PropertyKind,
    Command, ConfigProperty, MeasurementProperty, QmsVersion, 
    MachineSchema, StateProperty, StringMap,
};

#[derive(Debug, Clone, Deserialize)]
pub struct MachineSchemaRaw {
    pub qms_version: QmsVersion,
    pub name: String,
    pub schema_revision: u32,
    pub identification: MachineIdentification,

    #[serde(default)]
    pub config: StringMap<ConfigProperty>,

    #[serde(default)]
    pub state: StringMap<StateProperty>,

    #[serde(default)]
    pub measurements: StringMap<MeasurementProperty>,

    #[serde(default)]
    pub commands: StringMap<Command>,

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

    #[serde(default)]
    pub commands: StringMap<CommandDescriptions>,
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

        let mut commands = StringMap::new();
        for (name, descriptions) in raw.descriptions.commands {
            let Some(mut command) = raw.commands.shift_remove(&name) else {
                return Err(format!("description for unknown property: commands.{name}"));
            };

            command.description = descriptions.description;
            command.parameters = merge_with_properties(
                "commands", "commands",
                descriptions.parameters,
                command.parameters,
            )?;

            commands.insert(name, command);
        }
        
        Ok(Self {
            qms_version: raw.qms_version,
            name: raw.name,
            schema_revision: raw.schema_revision,
            identification: raw.identification,
            config,
            state,
            measurements,
            commands: raw.commands,
        })
    }
}

pub fn merge_with_properties<V>(
    section: &str,
    key: &str,
    descs: StringMap<DescriptionNode>,
    mut props: StringMap<Property<V>>,
) -> Result<StringMap<Property<V>>, String> {
    for (name, node) in descs {
        let Some(prop) = props.get_mut(&name) else {
            return Err(format!("description for unknown {section} property: {key}.{name}"));
        };

        match (&mut prop.kind, node) {
            (PropertyKind::Value(_), DescriptionNode::Leaf(desc)) => {
                prop.description = desc;
            }
            (PropertyKind::Group(children), DescriptionNode::Branch(child_descs)) => {
                let nested_key = format!("{key}.{name}");
                let merged = merge_with_properties(
                    section,
                    &nested_key,
                    child_descs,
                    std::mem::take(children),
                )?;
                *children = merged;
            }
            (PropertyKind::Value(_), DescriptionNode::Branch(_)) => {
                return Err(format!(
                    "expected a description for {section} property {key}.{name}, found a group"
                ));
            }
            (PropertyKind::Group(_), DescriptionNode::Leaf(_)) => {
                return Err(format!(
                    "expected a group of descriptions for {section} property {key}.{name}, found a single description"
                ));
            }
        }
    }

    Ok(props)
}
