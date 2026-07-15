use std::sync::Arc;
use axum::Router;
use crate::SharedState;

mod machines;

pub fn init_router() -> Router<Arc<SharedState>> {
    Router::new()
        .nest("/machines", machines::init_router())
}
