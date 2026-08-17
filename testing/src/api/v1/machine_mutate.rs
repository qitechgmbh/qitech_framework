use std::fmt::Debug;
use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_hub::ModuleContext;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub ident: MachineIdentificationUnique,
    pub data: serde_json::Value,
}

pub async fn post(
    State(ctx): State<Arc<ModuleContext>>,
    Json(body): Json<Request>,
) -> Response<Body> {
    ctx.request_tx.send(RuntimeRequest {
        request_id: 0,
        kind: RuntimeRequestKind::ExecuteCommand {
            target: body.ident,
            path: "hello.world".to_string(),
        },
    });

    (StatusCode::OK, ()).into_response()
}
