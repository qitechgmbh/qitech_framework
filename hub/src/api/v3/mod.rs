use std::sync::Arc;
use axum::{Router, routing};
use crate::SharedState;

mod machine_device_identification;
mod machines;

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .nest("/machines", machines::init_router())
        // .route("/machine_device_identification", routing::post())
}
