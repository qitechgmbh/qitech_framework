use std::collections::BTreeMap;
use std::sync::Arc;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use serde::Deserialize;
use tokio::sync::{mpsc, broadcast};
use clickhouse::{Client, Row};

use control_core::{
    schema,
    schema::latest::MachineSchema,
    RuntimeRequest,
    MachineIdentification, 
    RuntimeReport, 
};

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

mod migration;
pub use migration::migrate;

mod session;
pub use session::Session;
pub use session::EmbeddedSession;

mod tables;

mod ingest;
use ingest::IngestManager;

mod transaction;
use transaction::PendingRuntimeRequest;
use transaction::TransactionManager;

mod machine_registry;
use machine_registry::MachineRegistry;

use crate::session::SessionManager;

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

pub struct ControlHub {
    state: SharedState,

    // manages ingest and persisting of runtime reports
    ingest_manager: IngestManager,

    // manages requests sent via the api to the runtime
    transaction_manager: TransactionManager,
}

impl ControlHub {
    pub async fn init_embedded(
        config: Config,
        schemas: Vec<String>,
    ) -> anyhow::Result<(Self, EmbeddedSession)> {
        let incoming_schemas = schemas;

        let client = init_client(&config.db);
        let schemas = init_schema_registry(&client).await?;
        let machines = MachineRegistry::init(&client, &schemas).await?;

        // --- load schemas ---
        let schemas = import_schemas(schemas, incoming_schemas)?;

        // --- create channels ---
        let (data_tx, _) = broadcast::channel(64);
        let (pending_tx, pending_rx) = mpsc::channel(512);
        let (request_tx, request_rx) = mpsc::channel(512);

        // --- init state ---
        let state = SharedState {
            config,
            client,
            schemas: Arc::new(ArcSwap::new(Arc::new(schemas))),
            machines: Arc::new(ArcSwap::new(Arc::new(machines))),
            report_tx: data_tx.clone(),
            pending_tx,
        };

        // --- init managers ---
        let session_manager: SessionManager = todo!();
        let ingest_manager = IngestManager::init(&state);

        let transaction_manager = TransactionManager::init(
            &state, 
            pending_rx, 
            request_tx,
        ).await?;

        Ok((
            Self { 
                state,
                transaction_manager,
            },
            EmbeddedSession::new(data_tx, req_rx),
        ))
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let result = tokio::join!(
            self.ingest_manager.run(),
            self.transaction_manager.run(),

            // handles the client facing api
            // api::run(self.state.clone()),
        );

        // TODO: use this maybe?
        _ = result;

        Ok(())
    }
}

fn init_client(config: &DatabaseConfig) -> Client {
    let mut client = Client::default()
        .with_url(&config.url)
        .with_user(&config.user)
        // Enable JSON type and inserting JSON columns as string
        .with_setting("allow_experimental_json_type", "1")
        .with_setting("input_format_binary_read_json_as_string", "1");

    if let Some(password) = &config.password {
        client = client.with_password(password);
    }

    client.with_database(&config.name)
}

/// attempts to load the schema registry from the database
async fn init_schema_registry(
    client: &Client,
) -> anyhow::Result<Arc<BTreeMap<MachineIdentification, MachineSchema>>> {
    #[derive(Debug, Row, Deserialize)]
    struct SchemaRow {
        ident_vendor: u16,
        ident_machine: u16,
        content: String,
    }

    let fetched_schemas = client
        .query("SELECT * FROM machine_schemas")
        .fetch_all::<SchemaRow>()
        .await?;

    let mut registry = BTreeMap::new();

    for SchemaRow { content, .. } in fetched_schemas {
        let schema = schema::parse_latest(&content)?;

        if let Some(s) = registry.insert(schema.identification, schema) {
            return Err(anyhow!("Duplicate schema {}", s.identification));
        };
    }

    Ok(Arc::new(registry))
}

fn import_schemas(
    registry: Arc<BTreeMap<MachineIdentification, MachineSchema>>,
    incoming: Vec<String>,
) -> anyhow::Result<BTreeMap<MachineIdentification, MachineSchema>> {
    // lazily cloned copy-on-write map
    let mut new_schemas = (*registry).clone();

    for schema_data in incoming {
        let schema = schema::parse_latest(&schema_data)?;

        if let Some(s) = registry.get(&schema.identification) {
            if schema.schema_revision != s.schema_revision {
                return Err(anyhow!(
                    "schema revision mismatch for {}: expected {}",
                    schema.identification,
                    s.schema_revision,
                ));
            }

            // duplicate, continue
            continue;
        }

        // not duplicate, put into registry
        new_schemas.insert(schema.identification, schema);
    }

    Ok(new_schemas)
}
