use std::sync::Arc;
use clickhouse::{Client, Row};
use futures::TryFutureExt;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use axum::{Json, extract::{Path, State}};
use control_core::{MachineIdentificationUnique, ScalarValue, schema::v1_0::{ConfigProperty, PropertyKind}};
use crate::SharedState;

#[derive(Deserialize, Row)]
struct Response {
    
}

#[derive(Deserialize, Row)]
struct MeasurementSnapshot {
    value: Option<f64>,
}

pub(crate) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial)): Path<(String, u16)>,
) -> Result<Json<Response>, String> {
    // ensure we have such a machine type defined in the schemas
    let schemas = state.schemas.load();
    let Some((ident, schema)) = schemas.iter().find(|(_, s)| s.name == slug) else {
        return Err("No such machine slug".into());
    };

    let ident_unique = MachineIdentificationUnique {
        vendor: ident.vendor,
        machine: ident.machine,
        serial,
    };

    // ensure we have a machine registered that matches the request
    let machines = state.machines.load();
    if !machines.contains_key(&ident_unique) {
        return Err("No machine matches these parameters".into());
    }

    // --- walk schema to retrieve all properties ---

    let mut config = IndexMap::new();
    let mut path: String = "config".into();

    for (name, prop) in &schema.config {
        let path = format!("{path}.{name}");

        match &prop.kind {
            PropertyKind::Group(children) => {

            }

            PropertyKind::Value(_) => {
                let row = state.client
                    .query("SELECT value FROM measurements WHERE name EQ ? ORDER BY timestamp DESC LIMIT 1")
                    .bind(path)
                    .fetch_one::<MeasurementSnapshot>()
                    .map_err(|e| format!("{e}"))
                    .await?;

                config.insert(name, format!("{:#?}", row.value));
            }
        }
    }

    let value = state.client
        .query("SELECT value FROM measurements ORDER BY timestamp DESC LIMIT 1")
        .fetch_one::<MeasurementSnapshot>()
        .map_err(|e| format!("{e}"))
        .await?;

    todo!()
    // Ok(axum::Json(items))
}

#[derive(Deserialize, Row)]
struct ConfigRow { value: ScalarValue }

#[derive(Serialize)]
enum ConfigEntry {
    Group(IndexMap<String, ConfigEntry>),
    Value(ScalarValue)
}

async fn load_config(
    client: &Client,
    schema: &IndexMap<String, ConfigProperty>,
    path: &str,
) -> Result<IndexMap<String, ConfigEntry>, String> {
    let mut config = IndexMap::new();
    let mut stack = vec![("config".to_string(), schema)];

    while let Some((path, properties)) = stack.pop() {
        for (name, prop) in *properties {
            let child_path = format!("{path}.{name}");

            match &prop.kind {
                PropertyKind::Group(children) => {
                    stack.push((child_path, children));
                }

                PropertyKind::Value(v) => {
                    match v {
                        
                    }

                    let row = client
                        .query(
                            "SELECT value
                            FROM measurements
                            WHERE name = $name
                            ORDER BY timestamp DESC
                            LIMIT 1",
                        )
                        .bind(("name", child_path))
                        .fetch_one::<ConfigRow>()
                        .await
                        .map_err(|e| e.to_string())?;

                    config.insert(child_path, row.value);
                }
            }
        }
    }

    Ok(config)
}
