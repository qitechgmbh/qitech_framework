use std::{collections::{BTreeMap, HashMap}, sync::Arc};
use anyhow::bail;
use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use control_core::{MachineIdentification, MachineIdentificationUnique, ScalarValue, schema::{self, v1_0::{MachineSchema, Property, PropertyKind}}};
use indexmap::IndexMap;
use serde::Deserialize;
use crate::SchemaRegistry;

#[derive(Debug, Clone)]
pub struct MachineRegistry {
    inner: BTreeMap<MachineIdentificationUnique, MachineRegistryEntry>,
}

impl MachineRegistry {
    pub async fn init(
        client: &Client,
        schemas: &SchemaRegistry,
    ) -> anyhow::Result<Self> {
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

        let mut inner = BTreeMap::new();
        for IdentRow { identity, updated_at } in rows {
            let ident = MachineIdentificationUnique::from_u64(identity);

            if !schemas.contains_key(&MachineIdentification::from(ident)) {
                bail!("Could not find schema for registered machine {ident}");
            }

            let entry = MachineRegistryEntry {
                connected: false,
                updated_at,
                properties: Default::default(),
            };

            inner.insert(ident, entry);
        }

        Ok(Self { inner })
    }

    pub fn get_mut(
        &mut self, 
        ident: MachineIdentificationUnique
    ) -> Option<&mut MachineRegistryEntry> {
        self.inner.get_mut(&ident)
    }

    pub fn insert(
        &mut self,
        schemas: &Arc<BTreeMap<MachineIdentification, MachineSchema>>,
        ident: MachineIdentificationUnique
    ) -> anyhow::Result<()> {
        let Some(schema) = schemas.get(&MachineIdentification::from(ident)) else {
            // TODO: think about how to handle this error. Maybe disconnect from runtime ?
            bail!("Could not find schema for registered machine {ident}");
        };

        let entry = MachineRegistryEntry {
            connected: false,
            updated_at: Utc::now(),
            properties: Self::init_properties(schema),
        };

        self.inner.insert(ident, entry);
        Ok(())
    }

    /// Marks the machine as disconnected, no-op if 
    /// no such machine is present in the registry
    pub fn mark_connected(
        &mut self,
        schemas: &Arc<BTreeMap<MachineIdentification, MachineSchema>>,
        ident: MachineIdentificationUnique,
    ) -> anyhow::Result<()> {
        self.insert(schemas, ident)?;
        if let Some(entry) = self.inner.get_mut(&ident) {
            entry.connected = true;
        }

        Ok(())
    }

    /// Marks the machine as disconnected, no-op if 
    /// no such machine is present in the registry
    pub fn mark_disconnected(
        &mut self,
        ident: MachineIdentificationUnique
    ) {
        if let Some(entry) = self.inner.get_mut(&ident) {
            entry.connected = false;
            entry.properties.config.clear();
            entry.properties.state.clear();
            entry.properties.measurements.clear();
        }
    }

    // walks the schema the initialize the values with empty ones
    fn init_properties(
        schema: &MachineSchema,
    ) -> MachinePropertyCache {
        let mut properties = MachinePropertyCache::default();

        Self::walk(&schema.config, |path, v| {
            use schema::latest::config::Value::*;

            let value = match v {
                Enum(_) | String(_) => ScalarValue::String { value: None },
                Boolean(_) => ScalarValue::Boolean { value: None },
                Integer(_) => ScalarValue::Integer { value: None },
                Float(_) | Fraction(_) | Percentage(_) | Quantity { .. } => {
                    ScalarValue::Float { value: None }
                }
            };

            properties.config.insert(path, value);
        });

        Self::walk(&schema.state, |path, v| {
            use schema::latest::state::Value::*;

            let value = match v {
                Enum(_) | String(_) => ScalarValue::String { value: None },
                Boolean(_) => ScalarValue::Boolean { value: None },
                Integer(_) => ScalarValue::Integer { value: None },
                Float(_) | Fraction(_) | Percentage(_) | Quantity { .. } => {
                    ScalarValue::Float { value: None }
                }
            };

            properties.state.insert(path, value);
        });

        Self::walk(&schema.measurements, |path, _| {
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