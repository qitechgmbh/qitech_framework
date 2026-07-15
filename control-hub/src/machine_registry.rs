use std::collections::{BTreeMap, HashMap};
use anyhow::bail;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use control_core::{MachineIdentification, MachineIdentificationUnique, ScalarValue, schema::{self, v1_0::{MachineSchema, Property, PropertyKind}}};
use indexmap::IndexMap;
use serde::Deserialize;
use crate::{SchemaRegistry, SharedState};

pub type MachineRegistry = BTreeMap<MachineIdentificationUnique, MachineRegistryEntry>;

pub async fn init(
    client: &Client,
    schemas: &SchemaRegistry,
) -> anyhow::Result<MachineRegistry> {
    #[derive(Debug, Row, Deserialize)]
    struct IdentRow {
        identity: u64,
        updated_at: DateTime<Utc>,
    }

    let rows = client
        .query("SELECT
    identity,
    max(updated_at) AS updated_at
FROM machine_activity
GROUP BY identity"
        )
        .fetch_all::<IdentRow>()
        .await?;

    let mut registry = BTreeMap::new();
    for IdentRow { identity, updated_at } in rows {
        let ident = MachineIdentificationUnique::from_u64(identity);

        let Some(schema) = schemas.get(&MachineIdentification::from(ident)) else {
            bail!("Could not find schema for registered machine {ident}");
        };

        let entry = MachineRegistryEntry {
            connected: false,
            updated_at,
            properties: init_properties(schema),
        };

        registry.insert(ident, entry);
    }

    Ok(registry)
}

pub fn insert(
    zelf: &mut MachineRegistry,
    state: &SharedState,
    ident: MachineIdentificationUnique
) -> anyhow::Result<()> {
    let schemas = state.schemas.load();

    let Some(schema) = schemas.get(&MachineIdentification::from(ident)) else {
        // TODO: think about how to handle this error. Maybe disconnect from runtime ?
        bail!("Could not find schema for registered machine {ident}");
    };

    let entry = MachineRegistryEntry {
        connected: false,
        updated_at: Utc::now(),
        properties: init_properties(schema),
    };

    zelf.insert(ident, entry);
    Ok(())
}

/// Marks the machine as disconnected, no-op if 
/// no such machine is present in the registry
pub fn mark_disconnected(
    zelf: &mut MachineRegistry,
    ident: MachineIdentificationUnique
) {
    if let Some(entry) = zelf.get_mut(&ident) {
        entry.connected = false;
        entry.properties.config.clear();
        entry.properties.state.clear();
        entry.properties.measurements.clear();
    }
}

// walks the schema the initialize the values with empty ones
fn init_properties(schema: &MachineSchema) -> MachinePropertyCache {
    let mut properties = MachinePropertyCache::default();

    walk(&schema.config, |path, v| {
        use schema::latest::config::Value::*;

        let value = match v {
            Enum(_) | String(_) => ScalarValue::String(None),
            Boolean(_) => ScalarValue::Boolean(None),
            Integer(_) => ScalarValue::Integer(None),
            Float(_) | Fraction(_) | Percentage(_) | Quantity { .. } => {
                ScalarValue::Float(None)
            }
        };

        properties.config.insert(path, value);
    });

    walk(&schema.state, |path, v| {
        use schema::latest::state::Value::*;

        let value = match v {
            Enum(_) | String(_) => ScalarValue::String(None),
            Boolean(_) => ScalarValue::Boolean(None),
            Integer(_) => ScalarValue::Integer(None),
            Float(_) | Fraction(_) | Percentage(_) | Quantity { .. } => {
                ScalarValue::Float(None)
            }
        };

        properties.state.insert(path, value);
    });

    walk(&schema.measurements, |path, _| {
        properties.measurements.insert(path, None);
    });

    properties
}

fn walk<T>(
    root: &IndexMap<String, Property<T>>,
    mut visit: impl FnMut(String, &T),
) {
    let mut stack = vec![(String::new(), root)];

    while let Some((prefix, items)) = stack.pop() {
        for (name, prop) in items {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };

            match &prop.kind {
                PropertyKind::Group(children) => stack.push((path, children)),
                PropertyKind::Value(value) => visit(path, value),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MachineRegistryEntry {
    pub connected: bool,
    pub updated_at: DateTime<Utc>,
    pub properties: MachinePropertyCache,
}

#[derive(Debug, Clone, Default)]
pub struct MachinePropertyCache {
    pub config: HashMap<String, ScalarValue>,
    pub state: HashMap<String, ScalarValue>,
    pub measurements: HashMap<String, Option<f64>>,
}
