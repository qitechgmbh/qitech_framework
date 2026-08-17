use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::error::HelloMatchError;
use tokio::sync::oneshot;

use crate::MachineRegistry;
use crate::RuntimeRequestSender;
use crate::SchemaRegistry;
use crate::Swappable;

/// provider for making queries to retrieve data
pub trait QueryProvider: Send + Sync {}

#[derive(Debug, Clone)]
pub struct ActorContext {
    pub schemas: Swappable<SchemaRegistry>,
    pub machines: Swappable<MachineRegistry>,
    pub(crate) request_tx: RuntimeRequestSender,
}

impl ActorContext {
    pub async fn send_request(
        &self,
        request: RuntimeRequestKind,
    ) -> Result<Result<(), RuntimeRequestError>, oneshot::error::RecvError> {
        let (tx, rx) = oneshot::channel();
        self.request_tx
            .send((request, tx))
            .await
            .expect("transaction manager dropped request_tx");

        rx.await
    }
}

pub trait Actor: Send + Sync {
    fn run(self, ctx: ActorContext) -> impl Future<Output = ()> + Send + 'static;
}

type BoxFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

#[async_trait]
pub trait Listener: Send {
    async fn on_hello_rejected(&mut self, error: HelloMatchError) {
        _ = error;
    }

    async fn on_schema_sync(&mut self, schema: &MachineSchema) {
        _ = schema;
    }

    async fn on_schema_rejected(&mut self, reason: Arc<String>) {
        _ = reason;
    }

    async fn on_init_event_received(
        &mut self,
        event: Arc<RuntimeInitEvent>,
    ) {
        _ = event;
    }

    async fn on_report_received(
        &mut self,
        report: Arc<RuntimeReport>,
    ) {
        _ = report;
    }

    async fn on_transaction_completed(
        &mut self,
        request: Arc<RuntimeRequest>,
    ) {
        _ = request;
    }
}