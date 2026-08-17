use std::sync::Arc;

use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::session::error::HelloMatchError;
use tokio::sync::oneshot;

use crate::MachineRegistry;
use crate::RuntimeRequestSender;
use crate::SchemaRegistry;
use crate::Swappable;

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

pub trait Listener {
    /// Called when the runtime's hello message is rejected.
    async fn on_hello_rejected(error: HelloMatchError) {}

    /// Called when a schema is received for synchronization.
    ///
    /// The reactor may reject the schema.
    async fn on_schema_sync(schemas: Arc<SchemaRegistry>) {}

    /// Called when a schema is rejected.
    async fn on_schema_rejected(reason: Arc<String>) {}

    /// Called when an initialization event is received from the runtime.
    async fn on_init_event_received(event: Arc<RuntimeInitEvent>) {}

    /// Called when a report is received from the runtime.
    async fn on_report_received(report: Arc<RuntimeReport>) {}

    /// Called when a request is dispatched to the runtime.
    async fn on_transaction_completed(request: Arc<RuntimeRequest>) {}
}
