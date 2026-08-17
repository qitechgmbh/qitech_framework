use std::sync::Arc;

use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::session::error::HelloMatchError;

use crate::MachineRegistry;
use crate::RuntimeRequestSender;
use crate::SchemaRegistry;
use crate::Swappable;

pub struct RunnerContext {
    pub schemas: Swappable<SchemaRegistry>,
    pub machines: Swappable<MachineRegistry>,
    pub request_tx: RuntimeRequestSender,
}

pub trait Runner: Send + Sync {
    async fn run(self, ctx: RunnerContext);
}

pub trait Reactor {
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
