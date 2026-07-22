use std::sync::Arc;
use axum::{Router, routing};
use crate::SharedState;

mod history;

// -- router ---

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .route("/{property_name}/history", routing::get(history::get))
}
