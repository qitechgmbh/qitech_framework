use qitech_framework_hub::Actor;
use qitech_framework_hub::ActorContext;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::api::SharedState;
use crate::api::legacy::LegacySharedState;
use crate::api::legacy::init_socket_io;

pub struct ApiServer {
    state: SharedState,
    state_legacy: LegacySharedState,
}

impl ApiServer {
    pub fn new(state: SharedState, state_legacy: LegacySharedState) -> Self {
        Self {
            state,
            state_legacy,
        }
    }
}

impl Actor for ApiServer {
    async fn run(self, ctx: ActorContext) {
        let router = axum::Router::new()
            // .nest("/api/v1", v1::router())
            // .nest("/api/v2", v2::router())
            // .nest("/api/v3", v3::router())
            .layer(init_socket_io(
                self.state.clone(),
                self.state_legacy.clone(),
            ))
            .layer(CorsLayer::permissive())
            .with_state(ctx);

        //.nest("/api/v2", rest_api_router())
        //.layer(socketio_layer)
        //.layer(cors)
        //.layer(trace_layer);

        let listener = TcpListener::bind("0.0.0.0:3001")
            .await
            .expect("Failed to bind to port 3001");

        axum::serve(listener, router).await.unwrap();
    }
}
