use serde::Deserialize;
use serde::de::Error;

use crate::ident::MachineIdentification;
use crate::schema::MachineSchema;
use crate::schema::StringMap;
use crate::schema::Version;

pub type ParseError = yaml_serde::Error;

mod keyword;

mod tree;
use tree::DefinitionTree;
use tree::MetadataTree;

mod enum_variants;

mod version;
use version::VersionRaw;

mod config;
use config::ConfigPropertyInfoRaw;

mod state;
use state::StatePropertyDefinitionRaw;

mod measurements;
use measurements::MeasurementInfoRaw;

mod command;
use command::CommandDefinitionRaw;

mod events;
use events::EventInfoRaw;

pub fn parse_str(s: &str) -> Result<MachineSchema, ParseError> {
    let raw: MachineSchemaRaw = yaml_serde::from_str(s)?;

    let qms_version = Version {
        major: raw.qms_version.major,
        minor: raw.qms_version.minor,
    };

    if !Version::is_supported(qms_version) {
        return Err(ParseError::custom(format!(
            "Unsupported version: {}",
            qms_version
        )));
    }

    let name = raw.identification.name;
    let identification = MachineIdentification {
        vendor_id: raw.identification.vendor_id,
        machine_id: raw.identification.machine_id,
    };

    // --- construct config properties ---
    let mut config_properties = StringMap::new();

    for (name, info) in tree::collapse(raw.config) {
        config_properties.insert(name, info.0);
    }

    for (name, metadata) in tree::collapse(raw.metadata.config) {
        let Some(entry) = config_properties.get_mut(&name) else {
            return Err(ParseError::custom(format!(
                "metadata given for non-existent config property: {name}"
            )));
        };

        entry.metadata = metadata;
    }

    // --- construct state properties ---
    let mut state_properties = StringMap::new();

    for (name, info) in tree::collapse(raw.state) {
        state_properties.insert(name, info.0);
    }

    for (name, metadata) in tree::collapse(raw.metadata.state) {
        let Some(entry) = state_properties.get_mut(&name) else {
            return Err(ParseError::custom(format!(
                "metadata given for non-existent state property: {name}"
            )));
        };

        entry.metadata = metadata;
    }

    // --- construct measurements ---
    let mut measurements = StringMap::new();

    for (name, info) in tree::collapse(raw.measurements) {
        measurements.insert(name, info.0);
    }

    for (name, metadata) in tree::collapse(raw.metadata.measurements) {
        let Some(entry) = measurements.get_mut(&name) else {
            return Err(ParseError::custom(format!(
                "metadata given for non-existent measurement: {name}"
            )));
        };

        entry.metadata = metadata;
    }

    // --- construct commands ---
    let mut commands = StringMap::new();

    for (name, info) in tree::collapse(raw.commands) {
        commands.insert(name, info.0);
    }

    for (name, metadata) in tree::collapse(raw.metadata.commands) {
        let Some(entry) = commands.get_mut(&name) else {
            return Err(ParseError::custom(format!(
                "metadata given for non-existent command: {name}"
            )));
        };

        entry.metadata = metadata;
    }

    // --- construct events ---
    let mut events = StringMap::new();

    for (name, info) in tree::collapse(raw.events) {
        events.insert(name, info.0);
    }

    for (name, metadata) in tree::collapse(raw.metadata.events) {
        let Some(entry) = events.get_mut(&name) else {
            return Err(yaml_serde::Error::custom(format!(
                "metadata given for non-existent event: {name}"
            )));
        };

        entry.metadata = metadata;
    }

    // --- finish ---
    Ok(MachineSchema {
        qms_version,
        revision: raw.revision,
        name,
        identification,
        config_properties,
        state_properties,
        measurements,
        commands,
        events,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// Raw representation of the yaml doc
pub struct MachineSchemaRaw {
    // --- meta data ---
    pub qms_version: VersionRaw,
    pub revision: u32,

    // --- interface ---
    pub identification: IdentificationRaw,

    #[serde(default)]
    pub config: DefinitionTree<ConfigPropertyInfoRaw>,

    #[serde(default)]
    pub state: DefinitionTree<StatePropertyDefinitionRaw>,

    #[serde(default)]
    pub measurements: DefinitionTree<MeasurementInfoRaw>,

    #[serde(default)]
    pub commands: DefinitionTree<CommandDefinitionRaw>,

    #[serde(default)]
    pub events: DefinitionTree<EventInfoRaw>,

    #[serde(default)]
    pub metadata: ResourceMetadataRaw,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentificationRaw {
    pub name: String,
    pub vendor_id: u16,
    pub machine_id: u16,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceMetadataRaw {
    #[serde(default)]
    pub config: MetadataTree,

    #[serde(default)]
    pub state: MetadataTree,

    #[serde(default)]
    pub measurements: MetadataTree,

    #[serde(default)]
    pub commands: MetadataTree,

    #[serde(default)]
    pub events: MetadataTree,
}
