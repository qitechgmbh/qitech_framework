use std::{collections::HashMap, fmt::{self, Display, Formatter}};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;
use crate::QmsVersion;

// use crate::{
//     MachineIdentification,
//     schema::{AnyMachineSchema, QmsVersion}
// };

mod migration;

pub type Map<K, V> = IndexMap<K, V>;
pub type StringMap<T> = Map<String, T>;
pub type LocalizedText = Map<LanguageIdentifier, String>;

// pub type ConfigProperty = Property<config::Value>;
// pub type StateProperty = Property<state::Value>;
// pub type MeasurementProperty = Property<measurement::Value>;
// pub type CommandParameter = Property<command::ParameterValue>;
// pub use command::Command;

pub const VERSION: QmsVersion = QmsVersion { major: 1, minor: 0 };

// pub(crate) fn parse(data: &str) -> yaml_serde::Result<AnyMachineSchema> {
//     let schema = yaml_serde::from_str::<Schema>(data)?;
//     Ok(AnyMachineSchema::V1_0(schema))
// }

#[derive(Debug, Clone)]
pub struct Document {
    pub qms_version: QmsVersion,
    pub revision: u32,
    pub name: String,
    // pub identification: MachineIdentification,
    // pub config_properties: StringMap<ConfigProperty>,
    // pub state_properties: StringMap<StateProperty>,
    // pub measurements: StringMap<MeasurementProperty>,
    // pub commands: StringMap<CommandProperty>,
    // pub commands: StringMap<EventProperty>,
}

