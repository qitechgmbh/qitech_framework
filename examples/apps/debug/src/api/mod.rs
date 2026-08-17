use qitech_framework_hub::Actor;
use qitech_framework_hub::ActorContext;
use tokio::net::TcpListener;

mod adapter;
mod socketio;
mod v1;
// mod v2;
// mod v3;

pub struct ApiActor;

impl Actor for ApiActor {
    async fn run(self, ctx: ActorContext) {
        //let cors = CorsLayer::permissive();

        let router = axum::Router::new()
            .nest("/api/v1", v1::router())
            // .nest("/api/v2", v2::router())
            // .nest("/api/v3", v3::router())
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
