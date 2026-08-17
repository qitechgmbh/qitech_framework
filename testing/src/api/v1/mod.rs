use std::sync::Arc;

use axum::Router;
use axum::routing::post;
use qitech_framework_hub::ModuleContext;

mod machine_mutate;
mod write_machine_device_identification;

pub fn init_router() -> Router<Arc<ModuleContext>> {
    Router::new()
        .route(
            "/write_machine_device_identification",
            post(write_machine_device_identification::post),
        )
        .route("/machine/mutate", post(machine_mutate::post))
}
