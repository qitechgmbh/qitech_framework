use std::collections::BTreeMap;
use std::sync::Arc;
use arc_swap::ArcSwap;
use tokio::sync::{mpsc, broadcast};
use clickhouse::Client;

use control_core::{
    schema::latest::MachineSchema,
    RuntimeRequest,
    MachineIdentification, 
    RuntimeReport, 
};

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
