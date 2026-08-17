use std::pin::Pin;
use std::sync::Arc;

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

pub trait Listener: Send {
    fn on_hello_rejected<'a>(
        &'a mut self,
        error: HelloMatchError,
    ) -> BoxFuture<'a>;

    fn on_schema_sync<'a>(
        &'a mut self,
        schema: &'a MachineSchema,
    ) -> BoxFuture<'a>;

    fn on_schema_rejected<'a>(
        &'a mut self,
        reason: Arc<String>,
    ) -> BoxFuture<'a>;

    fn on_init_event_received<'a>(
        &'a mut self,
        event: Arc<RuntimeInitEvent>,
    ) -> BoxFuture<'a>;

    fn on_report_received<'a>(
        &'a mut self,
        report: Arc<RuntimeReport>,
    ) -> BoxFuture<'a>;

    fn on_transaction_completed<'a>(
        &'a mut self,
        request: Arc<RuntimeRequest>,
    ) -> BoxFuture<'a>;
}