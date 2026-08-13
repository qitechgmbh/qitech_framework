use std::collections::BTreeMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use clickhouse::Client;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::MachineSchema;
use qitech_framework_core::RuntimeReport;
use qitech_framework_core::RuntimeRequest;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

pub mod migration;

mod tables;

mod ingest;
use ingest::IngestManager;

mod transaction;
use transaction::PendingRuntimeRequest;
use transaction::TransactionManager;

mod machine_registry;
use machine_registry::MachineRegistry;

mod utils;

mod embedded;
pub use embedded::Embedded;
pub use embedded::EmbeddedSession;

mod connection;
mod standalone;

type Swappable<T> = Arc<ArcSwap<T>>;
type SchemaRegistry = BTreeMap<MachineIdentification, MachineSchema>;

type RuntimeReportSender = broadcast::Sender<Arc<RuntimeReport>>;
type RuntimeReportReceiver = broadcast::Receiver<Arc<RuntimeReport>>;

type RuntimeRequestSender = mpsc::Sender<RuntimeRequest>;
type RuntimeRequestReceiver = mpsc::Receiver<RuntimeRequest>;

#[derive(Clone)]
struct SharedState {
    pub config: Config,
    pub client: Client,
    pub schemas: Swappable<SchemaRegistry>,
    pub machines: Swappable<MachineRegistry>,

    // fan out of runtime reports to sub systems
    pub report_tx: RuntimeReportSender,

    /// channel to start a transaction
    pub pending_tx: mpsc::Sender<PendingRuntimeRequest>,
    // TODO: implemen
    // pub runtime_state: RuntimeState
}
