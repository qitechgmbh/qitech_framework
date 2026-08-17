use axum::routing::post;
use qitech_framework_hub::Module;
use qitech_framework_hub::ModuleContext;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

// mod machines;
// mod socket_io;

mod v1;

struct FrontendModule;

impl Module for FrontendModule {
    async fn run(self, mut ctx: ModuleContext) {
        let cors = CorsLayer::permissive();

        let router = axum::Router::new()
            .route(
                "/api/v1/write_machine_device_identification",
                post(post_write_machine_device_identification),
            )
            // .route("/api/v1/machine/mutate", post(post_machine_mutate))
            // .nest("/api/v2", rest_api_router())
            // .layer(socketio_layer)
            .layer(cors)
            // .layer(trace_layer)
            ;

        let listener = TcpListener::bind("0.0.0.0:3001")
            .await
            .expect("Failed to bind to port 3001");

        axum::serve(listener, router).await.unwrap();
    }
}
