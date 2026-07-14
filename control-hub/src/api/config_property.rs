use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::{Json, extract::{Path, State}};
use control_core::{ConfigMutationOrigin, MachineIdentificationUnique};
use control_core::schema::latest::{PropertyKind, config::ValueV2};
use crate::SharedState;

pub(crate) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property_name)): Path<(String, u16, String)>,
) -> Result<Json<Vec<Entry>>, String> {
    // ensure we have such a machine type defined in the schemas
    let schemas = state.schemas.load();
    let Some((ident, schema)) = schemas.iter().find(|(_, s)| s.name == slug) else {
        return Err("No such machine slug".into());
    };

    let path: Vec<&str> = property_name.split('.').collect();

    let mut current = &schema.config;

    for (index, part) in path.iter().enumerate() {
        let Some(prop) = current.get(*part) else {
            println!("not found at {}", part);
            break;
        };

        if index == path.len() - 1 {
            match prop.kind {
                PropertyKind::Group(_) => return Err("Not a value".into()),
                PropertyKind::Value(info) => {
                    match info {
                        ValueV2::Enum(_) => {
                            let sql = "SELECT ";
                            state.client.query(sql).fetch_all();
                        },
                        ValueV2::String(_) => {

                        },
                        ValueV2::Boolean(_) => {
                            let sql = "SELECT timestamp, value_bool WHERE identity = ? ORDER BY timestamp  ";
                            state.client.query(sql).fetch_all();
                        },
                        ValueV2::Integer(_) => {

                        },
                        ValueV2::Float(_) => {

                        },
                        ValueV2::Quantity { unit, .. } => {

                        },
                    }
                },
            }

            // Found the final property
            println!("found: {:?}", prop);
            break;
        }

        match &prop.kind {
            PropertyKind::Group(children) => current = children,
            PropertyKind::Value(_) => {
                println!("{} is not a group", part);
                break;
            }
        }
    }

    Ok(axum::Json(items))
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Entry {
    timestamp: DateTime<Utc>,
    origin: ConfigMutationOrigin,
}
