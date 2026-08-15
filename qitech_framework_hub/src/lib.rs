use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use arc_swap::ArcSwap;
use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexMap;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EventRecord;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::schema::FloatSemantic;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::schema::ScalarPropertyKind;
use qitech_framework_core::session::AsyncControllerTransport;
use qitech_framework_core::session::controller_async::SessionHandshake;
use qitech_framework_core::session::controller_async::SessionRunning;
use qitech_framework_core::session::error::SchemaSyncError;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

/*
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
*/

mod modules;
pub use modules::Module;
pub use modules::ModuleContext;
use tokio::task::JoinSet;

type Lockable<T> = Arc<RwLock<T>>;
type Swappable<T> = Arc<ArcSwap<T>>;

pub type SchemaRegistry = BTreeMap<MachineIdentification, MachineSchema>;
pub type MachineRegistry = BTreeMap<MachineIdentificationUnique, MachineEntry>;

type RuntimeReportSender = broadcast::Sender<Arc<RuntimeReport>>;
type RuntimeReportReceiver = broadcast::Receiver<Arc<RuntimeReport>>;

type RuntimeRequestSender = mpsc::Sender<RuntimeRequest>;
type RuntimeRequestReceiver = mpsc::Receiver<RuntimeRequest>;

// database -> receive query, receive transaction etc

// Hub responsibilities:
// Request Manager -> api module can call into it -> make request via oneshot and receive result
// Database -> receives data like every other module / but how to handle queries

type ModuleFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct HubConfiguration {
    report_tx: RuntimeReportSender,
    request_tx: RuntimeRequestSender,
    request_rx: RuntimeRequestReceiver,

    machines: Lockable<MachineRegistry>,
    schemas: Swappable<SchemaRegistry>,
    modules: Vec<ModuleFuture>,
}

impl HubConfiguration {
    pub fn new() -> Self {
        let (report_tx, _) = broadcast::channel(32);
        let (request_tx, request_rx) = mpsc::channel(128);

        Self {
            report_tx,
            request_tx,
            request_rx,
            machines: Default::default(),
            schemas: Default::default(),
            modules: Default::default(),
        }
    }

    pub fn module<M: Module + 'static>(mut self, module: M) -> Self {
        let ctx = ModuleContext {
            schemas: self.schemas.clone(),
            machines: self.machines.clone(),
            report_rx: self.report_tx.subscribe(),
            request_tx: self.request_tx.clone(),
        };

        self.modules.push(Box::pin(module.run(ctx)));
        self
    }
}

impl Default for HubConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run<T: AsyncControllerTransport + 'static>(
    config: HubConfiguration,
    session: SessionHandshake<T>,
) -> Result<(), i64> {
    // TODO: load from storage
    // TODO: allow to restore session back to handshake

    // --- process hello ---
    let session = session.complete().await.unwrap();

    // --- sync schemas ---
    let mut schemas = SchemaRegistry::new();

    let session = session
        .sync(|schema| {
            if schemas.insert(schema.identification, schema).is_some() {
                return Err(SchemaSyncError::DuplicateItem);
            }

            Ok(())
        })
        .await
        .unwrap();

    config.schemas.store(Arc::new(schemas));

    // --- receive init events ---
    let session = session
        .complete(|event| {
            _ = event;
            Ok(()) // Why do I need result here ? 
        })
        .await
        .unwrap();

    // --- start tasks ---
    let mut tasks = JoinSet::new();

    // --- ingest for receiving data ---
    tasks.spawn(session_manager(
        session,
        config.report_tx,
        config.request_rx,
    ));

    // --- user provided modules ---
    for module in config.modules {
        tasks.spawn(module);
    }

    // --- wait for tasks ---
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(()) => {}
            Err(error) => {
                // Task panicked or was cancelled.
                eprintln!("task failed: {error}");
            }
        }
    }

    Ok(())
}

// TODO: handle lifecycle of the connection
async fn session_manager<T: AsyncControllerTransport>(
    mut session: SessionRunning<T>,
    report_sender: RuntimeReportSender,
    mut request_receiver: RuntimeRequestReceiver,
) {
    loop {
        tokio::select! {
            biased;

            report = session.recv_report() => {
                let report = report.unwrap();
                report_sender.send(Arc::new(report)).unwrap();
            }

            request = request_receiver.recv() => {
                let request = request.unwrap();
                session.send_request(request).await.unwrap();
            }
        }
    }
}

async fn request_manager<T: AsyncControllerTransport>(
    mut session: SessionRunning<T>,
    report_sender: RuntimeReportSender,
) {
    loop {
        let report = session.recv_report().await.unwrap();
        report_sender.send(Arc::new(report)).unwrap();
    }
}

async fn api<T: AsyncControllerTransport>(mut session: SessionRunning<T>) {
    loop {
        let report = session.recv_report().await.unwrap();
        println!("report: {report:#?}");
    }
}

#[derive(Clone)]
struct SharedState {
    pub schemas: Swappable<SchemaRegistry>,
    pub machines: Arc<RwLock<MachineRegistry>>,

    // fan out of runtime reports to sub systems
    pub report_tx: RuntimeReportSender,
    // /// channel to start a transaction
    // // pub pending_tx: mpsc::Sender<PendingRuntimeRequest>,
    // // TODO: implemen
    // // pub runtime_state: RuntimeState
}

pub struct MachineEntry {
    pub updated_at: DateTime<Utc>,
    pub config_props: IndexMap<String, ConfigPropertyEntry>,
    pub state_props: IndexMap<String, StatePropertyEntry>,
    pub measurements: IndexMap<String, MeasurementEntry>,
}

pub struct ConfigPropertyEntry {
    pub kind: ScalarPropertyKind,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,

    pub value: ScalarValue,
    pub default: ScalarValue,
    pub capability: OperationCapability,
    pub constraints: Constraints,
}

pub struct StatePropertyEntry {
    pub kind: ScalarPropertyKind,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,
    pub value: ScalarValue,
}

pub struct MeasurementEntry {
    pub label: String,
    pub value: Option<f64>,
    pub repr: FloatSemantic,
}
