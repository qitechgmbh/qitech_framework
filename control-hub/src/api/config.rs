use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::{Json, extract::State};
use control_core::{MachineIdentification, vendors};
use crate::SharedState;

pub(crate) async fn get(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<Vec<Entry>>, String> {
    let mut items: Vec<Entry> = Vec::new();

    for (ident_unique, (last_active, connected)) in state.machines.load().iter() {
        let ident = MachineIdentification::from(*ident_unique);
        let schemas = state.schemas.load();

        let name = if let Some(v) = schemas.get(&ident) {
            v.name.clone()
        } else { "N/A".into() };

        items.push(Entry {
            name,
            vendor: vendors::get_by_id(ident.vendor).unwrap_or("N/A"),
            serial: ident_unique.serial,
            connected: *connected,
            last_active: *last_active,
        });
    }

    Ok(axum::Json(items))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Entry {
    name: String,
    vendor: &'static str,
    serial: u16,
    connected: bool,
    last_active: DateTime<Utc>,
}
