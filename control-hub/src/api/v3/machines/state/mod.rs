use std::sync::Arc;
use serde::Serialize;
use axum::{Json, Router, extract::{Path, State}, routing};
use control_core::schema::latest::{Unit, state};

use crate::SharedState;
use crate::api::common::{ApiError, get_machine_info, get_property_info};

// mod history;

// -- router ---

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/{property_name}",routing::get(get))
        // .route("/{property_name}/history", routing::get(history::get))
}

// --- GET --- 

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum GetResponse {
    Enum { value: Option<String> },
    String { value: Option<String> },
    Boolean { value: Option<bool> },
    Integer { value: Option<i64> },
    Float { value: Option<f64> },
    Percentage { value: Option<f64> },
    Fraction { value: Option<f64> },
    Quantity {
        unit: Unit, 
        value: Option<f64>,
    },
}

pub(super) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property_name)): Path<(String, u16, String)>,
) -> Result<Json<GetResponse>, ApiError> {
    // get schema info
    let schemas = state.schemas.load();
    let (ident, schema) = get_machine_info(&schemas, &slug, serial)?;
    let prop_info = get_property_info(&schema.state, &property_name)?;

    // get value
    let machines = state.machines.load();
    let props = &machines.get(&ident).expect("must exist").properties.state;
    let value = props.get(&property_name).expect("must exist").clone();

    let response = match prop_info {
        state::Value::Enum(_) => GetResponse::Enum { value: value.string() },
        state::Value::String(_) => GetResponse::String { value: value.string() },
        state::Value::Boolean(_) => GetResponse::Boolean { value: value.boolean() },
        state::Value::Integer(_) => GetResponse::Integer { value: value.integer() },
        state::Value::Float(_) => GetResponse::Float { value: value.float() },
        state::Value::Fraction(_) => GetResponse::Fraction { value: value.float() },
        state::Value::Percentage(_) => GetResponse::Percentage { value: value.float() },
        state::Value::Quantity { unit, .. } => GetResponse::Quantity { 
            unit: *unit, 
            value: value.float(),
        },
    };

    Ok(axum::Json(response))
}
