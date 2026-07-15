use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use axum::{Json, Router, extract::State, routing};

use control_core::{MachineIdentification, vendors};
use crate::SharedState;

mod config;
mod state;
mod measurements;

// -- router ---

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/", routing::get(get))
        .nest("/{slug}/{serial}/config", config::init_router())
        .nest("/{slug}/{serial}/state", state::init_router())
        .nest("/{slug}/{serial}/measurements", measurements::init_router())
}

// --- GET ---

pub(crate) async fn get(
    State(state): State<Arc<SharedState>>,
) -> Result<Json<Vec<MachineInfo>>, String> {
    let mut items: Vec<MachineInfo> = Vec::new();

    for (ident_unique, entry) in state.machines.load().iter() {
        let ident = MachineIdentification::from(*ident_unique);
        let schemas = state.schemas.load();

        let name = if let Some(v) = schemas.get(&ident) {
            v.name.clone()
        } else { "N/A".into() };

        items.push(MachineInfo {
            name,
            vendor: vendors::get_by_id(ident.vendor).unwrap_or("N/A"),
            serial: ident_unique.serial,
            connected: entry.connected,
            last_active: entry.updated_at,
        });
    }

    Ok(axum::Json(items))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineInfo {
    name: String,
    vendor: &'static str,
    serial: u16,
    connected: bool,
    last_active: DateTime<Utc>,
}
