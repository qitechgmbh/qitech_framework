use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use crate::SharedState;

mod types;
pub use types::TransactionId;
pub use types::RuntimeTransaction;

mod common;
mod v3;

pub(crate) async fn run(state: SharedState) -> anyhow::Result<()> {
    let api_address = &state.config.api_address;

    let cors = CorsLayer::permissive();
    _ = cors; // TODO: enable

    // TODO: add v1 and v2 from old control for compat and mark as deprecated
    let router = axum::Router::new()
        // .nest("/api/v1", init_router_v1())
        // .nest("/api/v2", init_router_v2())
        .nest("/api/v3", v3::init_router())
        // .layer(cors)
        .with_state(Arc::new(state.clone()));

    let listener = TcpListener::bind(&api_address).await?;
    println!("[RestApi] Listening on {}", &api_address);

    axum::serve(listener, router).await?;
    Ok(())
}
