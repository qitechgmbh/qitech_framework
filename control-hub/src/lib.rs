use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use anyhow::bail;
use chrono::{DateTime, Utc};
use control_core::ScalarValue;
use control_core::schema::v1_0::PropertyKind;
use serde::Deserialize;
use arc_swap::ArcSwap;
use tokio::spawn;
use tokio::sync::oneshot;
use tokio::sync::{mpsc, broadcast, watch};
use clickhouse::{Client, Row};

use control_core::{
    schema,
    MachineIdentification, MachineIdentificationUnique, 
    RuntimeExport, schema::latest::MachineSchema,
};

mod tasks;
use tasks::sync_machine_registry;

mod migration;
pub use migration::migrate;

mod session;

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

mod embedded;
pub use embedded::EmbeddedSession;
use tokio::time::sleep;

pub mod api;
use crate::api::RuntimeRequest;

pub mod export_processor;

mod machine_registry;
use machine_registry::MachineRegistry;
use machine_registry::MachineRegistryEntry;
use machine_registry::MachinePropertyCache;

mod exporter;

type Swappable<T> = Arc<ArcSwap<T>>;
type SchemaRegistry = BTreeMap<MachineIdentification, MachineSchema>;

#[derive(Clone)]
struct SharedState {
    /// config ... as the name suggests
    pub config: Config,

    /// database client
    pub client: Client,

    /// schema registry
    pub schemas: Swappable<SchemaRegistry>,

    /// machine registry
    pub machines: Swappable<MachineRegistry>,

    /// channel for forwarding data exports
    pub data_tx: broadcast::Sender<Arc<RuntimeExport>>,

    /// channel for receiving requests
    pub req_tx: mpsc::Sender<(RuntimeRequest, oneshot::Sender<Result<(), String>>)>,

    /// shutdown signal receiver
    pub shutdown_rx: watch::Receiver<()>,
}

pub struct ControlHub {
    state: SharedState,
}

impl ControlHub {
    pub async fn init_embedded(
        config: Config,
        schemas: Vec<String>,
        shutdown_rx: watch::Receiver<()>,
    ) -> anyhow::Result<(Self, EmbeddedSession)> {
        let incoming_schemas = schemas;

        let client = init_client(&config.database);
        let schemas = init_schema_registry(&client).await?;
        let machines = machine_registry::init(&client, &schemas).await?;

        // try to import schemas of the runtime
        let schemas = import_schemas(schemas, incoming_schemas)?;

        let (data_tx, _) = broadcast::channel(64);
        let (req_tx, req_rx) = mpsc::channel(1024);

        spawn(async {
            // keep alive for now, TODO: use it ...
            let x = req_rx;
            loop {
                sleep(Duration::from_secs(10)).await;
            }
        });

        let state = SharedState {
            config,
            client,
            schemas: Arc::new(ArcSwap::new(Arc::new(schemas))),
            machines: Arc::new(ArcSwap::new(Arc::new(machines))),
            data_tx: data_tx.clone(),
            req_tx,
            shutdown_rx,
        };

        Ok((
            Self { state },
            EmbeddedSession::new(data_tx),
        ))
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // simply start all processes
        let result = tokio::join!(
            sync_machine_registry::run(self.state.clone()),
            exporter::run(self.state.clone()),
            api::run(self.state.clone()),
        );

        // TODO: use this maybe?
        _ = result;

        Ok(())
    }
}

fn init_client(config: &DatabaseConfig) -> Client {
    let mut client = Client::default()
        .with_url(&config.url)
        .with_user(&config.user);

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
        content: String,
    }

    let fetched_schemas = client
        .query("SELECT * FROM machine_schemas")
        .fetch_all::<SchemaRow>()
        .await?;

    let mut registry = BTreeMap::new();

    for SchemaRow { content } in fetched_schemas {
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

// --- misc --- 
#[derive(Debug, Clone)]
pub struct MachineRegistryEntry {
    online: bool,
    last_online: DateTime<Utc>,
    properties: MachinePropertyCache,
}

#[derive(Debug, Clone, Default)]
pub struct MachinePropertyCache {
    config: HashMap<String, ScalarValue>,
    state: HashMap<String, ScalarValue>,
    measurements: HashMap<String, Option<f64>>,
}
