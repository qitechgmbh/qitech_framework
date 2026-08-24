use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_hub::Actor;
use qitech_framework_hub::ActorContext;
use qitech_framework_hub::Listener;
use tokio::net::TcpListener;

mod adapter;
mod socketio;
mod v1;
// mod v2;
// mod v3;

// -> Query Provider: query().config()

// What does db want to persist -> init event, transactions, reports
// event log: runtime send hello, init event:
// but also capture the outgoing messages

pub struct SharedState {
    adapters: HashMap<i64, i64>,
}

struct MachineInstance {
    // pub config: IndexMap<String, ConfigField>,
}

trait MachineStreamAdapter {
    fn init_measurements_event(instance: &MachineInstance);
    fn init_state_event(instance: &MachineInstance);
}

pub struct ApiListener;

#[async_trait]
impl Listener for ApiListener {
    async fn on_init_event_received(&mut self, event: Arc<RuntimeInitEvent>) {
        _ = event;
        println!("RECEIVED EVENT");
    }

    async fn on_report_received(&mut self, report: Arc<RuntimeReport>) {
        for snapshot in &report.machines.measurement_snapshots {
            
        }

        _ = report;
        println!("RECEIVED REPORT");
    }
}

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
