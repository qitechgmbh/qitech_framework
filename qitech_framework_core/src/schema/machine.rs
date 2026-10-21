use kdl::KdlDocument;
use serde::Deserialize;
use serde::Serialize;

use crate::schema::SchemaError;
use crate::schema::ValueDefinition;
use crate::schema::error::Issues;
use crate::schema::parser;

// --- machine schema ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineSchema {
    /// Imported unit catalogs, e.g. `si`.
    pub imports: Vec<String>,
    /// Machine type name, e.g. `winder_v1`.
    pub name: String,
    pub config: Vec<ValueDefinition>,
    pub state: Vec<ValueDefinition>,
    pub measurements: Vec<ValueDefinition>,
    pub inputs: Vec<ValueDefinition>,
    pub commands: Vec<CommandDefinition>,
    pub events: Vec<EventDefinition>,
}

impl MachineSchema {
    pub fn parse_str(input: &str) -> Result<Self, SchemaError> {
        let document: KdlDocument = input
            .parse()
            .map_err(|e: kdl::KdlError| SchemaError::from_kdl(input, e))?;

        let mut issues = Issues::new(input);
        let schema = parser::parse_schema(&document, &mut issues);
        issues.finish(schema)
    }
}

// --- command ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Dotted path within `commands`, e.g. `traverse.goto_home`.
    pub path: String,
    pub kind: CommandKind,
    pub params: Vec<ValueDefinition>,
    pub returns: Vec<ValueDefinition>,
    pub errors: Vec<ErrorDefinition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind {
    /// Bounded action that completes, may take parameters.
    Action,
    /// State transition, never takes parameters.
    Transition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDefinition {
    pub name: String,
    pub fields: Vec<ValueDefinition>,
}

// --- event ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDefinition {
    /// Dotted path within `events`, e.g. `traverse.homing_failed`.
    pub path: String,
    pub fields: Vec<ValueDefinition>,
}
