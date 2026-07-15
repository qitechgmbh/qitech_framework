use std::sync::Arc;
use axum::{Json, Router, extract::State, routing};
use crate::SharedState;

// Compatibility module for V1 required by the current frontend

// --- router ---

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/write_machine_device_identification", routing::get(get).put(put))
        .route("/machine/mutate", routing::get(history::get))
}

// --- mutate ---



// .route(
//     "/api/v1/write_machine_device_identification",
//     post(post_write_machine_device_identification),
// )
// .route("/api/v1/machine/mutate", post(post_machine_mutate))