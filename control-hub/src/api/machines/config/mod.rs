use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing};

use control_core::ScalarValue;
use control_core::schema::latest::{Unit, config};

use crate::SharedState;
use crate::api::RuntimeRequest;
use crate::api::common::{ApiError, get_machine_info, get_property_info};

mod history;

// -- router ---

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/{property_name}", routing::get(get).put(put))
        .route("/{property_name}/history", routing::get(history::get))
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
    }
}

pub(super) async fn get(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, property_name)): Path<(String, u16, String)>,
) -> Result<Json<GetResponse>, ApiError> {

    // get schema info
    let schemas = state.schemas.load();
    let (ident, schema) = get_machine_info(&schemas, &slug, serial)?;
    let prop_info = get_property_info(&schema.config, &property_name)?;

    // get value
    let props = state.properties.load();
    let props = props.config.get(&ident).expect("must exist");
    let value = props.get(&property_name).expect("must exist").clone();

    use config::Value::*;
    let response = match prop_info {
        Enum(_) => GetResponse::Enum { value: value.string() },
        String(_) => GetResponse::String { value: value.string() },
        Boolean(_) => GetResponse::Boolean { value: value.boolean() },
        Integer(_) => GetResponse::Integer { value: value.integer() },
        Float(_) => GetResponse::Float { value: value.float() },
        Fraction(_) => GetResponse::Fraction { value: value.float() },
        Percentage(_) => GetResponse::Percentage { value: value.float() },
        Quantity { unit, .. } => GetResponse::Quantity { 
            unit: *unit, 
            value: value.float(),
        },
    };

    Ok(axum::Json(response))
}

// --- PUT ---

#[derive(Debug, Deserialize)]
pub struct PutRequest {
    pub value: serde_json::Value,
}

pub(super) async fn put(
    State(state): State<Arc<SharedState>>,
    Path((slug, serial, name)): Path<(String, u16, String)>,
    Json(body): Json<PutRequest>,
) -> Result<StatusCode, ApiError> {
    // get schema info
    let schemas = state.schemas.load();
    let (ident, schema) = get_machine_info(&schemas, &slug, serial)?;
    let prop_info = get_property_info(&schema.config, &name)?;

    let value = match prop_info {
        config::Value::Enum(info) => {
            let expected = format!("expected one of [{}]", info.variants.list().join(", "));

            let value = body
                .value
                .as_str()
                .ok_or_else(|| bad_request(expected.clone()))?;

            let Some(v) = info.variants.get_int(value) else {
                return Err(bad_request(expected));
            };

            ScalarValue::Integer(Some(v))
        }

        config::Value::String(info) => {
            let value = body.value.as_str().map(str::to_string);

            if !info.nullable && value.is_none() {
                return Err(bad_request("value cannot be null"));
            }

            if let Some(v) = &value {
                if !info.length.in_range(v.len() as u32) {
                    return Err(unprocessable("string length out of bounds"));
                }
            }

            ScalarValue::String(value)
        }

        config::Value::Boolean(info) => {
            let value = body.value.as_bool();

            if !info.nullable && value.is_none() {
                return Err(bad_request("value cannot be null"));
            }

            ScalarValue::Boolean(value)
        }

        config::Value::Integer(info) => {
            let value = body.value.as_i64();

            if !info.nullable && value.is_none() {
                return Err(bad_request("value cannot be null"));
            }

            if let Some(v) = value {
                if !info.range.in_range(v) {
                    return Err(unprocessable("value out of bounds"));
                }
            }

            ScalarValue::Integer(value)
        }

        config::Value::Float(info)
        | config::Value::Fraction(info)
        | config::Value::Percentage(info) => {
            let value = body.value.as_f64();

            if !info.nullable && value.is_none() {
                return Err(bad_request("value cannot be null"));
            }

            if let Some(v) = value {
                if !info.range.in_range(v) {
                    return Err(unprocessable("value out of bounds"));
                }
            }

            ScalarValue::Float(value)
        }

        config::Value::Quantity { value, .. } => {
            let info = value;
            let value = body.value.as_f64();

            if !info.nullable && value.is_none() {
                return Err(bad_request("value cannot be null"));
            }

            if let Some(v) = value {
                if !info.range.in_range(v) {
                    return Err(unprocessable("value out of bounds"));
                }
            }

            ScalarValue::Float(value)
        }
    };

    let (tx, rx) = oneshot::channel();
    _ = rx;

    let request = RuntimeRequest::MutateConfig { ident, name, value };

    println!("Received config mutation request: {request:#?}");

    state.req_tx
        .send((request, tx))
        .await
        .map_err(|_| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "runtime unavailable".into(),
        })?;

    // TODO: enable
    // // wait for response
    // let Ok(result) = rx.await else {
    //     // sub system crashed/terminated
    //     return Err("Internal Error".into());
    // };


    Ok(StatusCode::NO_CONTENT)
}

fn bad_request(message: impl Into<String>) -> ApiError {
    ApiError {
        status: StatusCode::BAD_REQUEST,
        message: message.into(),
    }
}

fn unprocessable(message: impl Into<String>) -> ApiError {
    ApiError {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        message: message.into(),
    }
}
