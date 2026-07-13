use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::{Json, extract::{Path, State}};
use control_core::MachineIdentification;
use crate::{vendors, SharedState};

pub(crate) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((name, serial)): Path<(String, u32)>,
) -> Result<Json<Vec<Entry>>, String> {
    

    let mut items: Vec<Entry> = Vec::new();

    for (ident_unique, connected) in state.machines.load().iter() {
        let ident = MachineIdentification::from(*ident_unique);
        let schemas = state.schemas.load();

        let name = if let Some(v) = schemas.get(&ident) {
            v.name.clone()
        } else { "N/A".into() };

        items.push(Entry {
            name,
            vendor: vendors::get(ident.vendor).unwrap_or("N/A"),
            serial: ident_unique.serial,
            connected: *connected,
            last_active: Utc::now(),
        });
    }

    Ok(axum::Json(items))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Entry {
    name: String,
    vendor: &'static str,
    serial: u32,
    connected: bool,
    last_active: DateTime<Utc>,
}
