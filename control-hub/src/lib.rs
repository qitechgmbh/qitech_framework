use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use control_core::ScalarValue;
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

mod exporter;

type Swappable<T> = Arc<ArcSwap<T>>;

#[derive(Clone)]
struct SharedState {
    /// config ... as the name suggests
    pub config: Config,

    /// database client
    pub client: Client,

    /// schema registry
    pub schemas: Swappable<BTreeMap<MachineIdentification, MachineSchema>>,

    /// machine registry (ident, connected yes/no)
    pub machines: Swappable<BTreeMap<MachineIdentificationUnique, (DateTime<Utc>, bool)>>,

    pub properties: Swappable<PropertyCache>,

    /// channel for forwarding data exports
    pub data_tx: broadcast::Sender<Arc<RuntimeExport>>,

    /// channel for receiving requests
    pub req_tx: mpsc::Sender<(RuntimeRequest, oneshot::Sender<Result<(), String>>)>,

    /// shutdown signal receiver
    pub shutdown_rx: watch::Receiver<()>,
}

#[derive(Clone, Default)]
pub struct PropertyCache {
    config: HashMap<MachineIdentificationUnique, HashMap<String, ScalarValue>>,
    state: HashMap<MachineIdentificationUnique, HashMap<String, ScalarValue>>,
    measurements: HashMap<MachineIdentificationUnique, HashMap<String, f64>>,
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
        let client = init_client(&config.database);
        let schema_registry = init_schema_registry(&client).await?;
        let machines = init_machine_registry(&client).await?;

        // try to import schemas of the runtime
        let schemas = import_schemas(schema_registry, schemas)?;

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
            schemas: Arc::new(ArcSwap::new(schemas)),
            machines: Arc::new(ArcSwap::new(machines)),
            properties: Arc::new(ArcSwap::new(Default::default())),
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

async fn init_machine_registry(
    client: &Client,
) -> anyhow::Result<Arc<BTreeMap<MachineIdentificationUnique, (DateTime<Utc>, bool)>>> {
    #[derive(Debug, Row, Deserialize)]
    struct IdentRow {
        vendor: u16,
        machine: u16,
        serial: u16,
        last_active: DateTime<Utc>,
    }

    let rows = client
        .query("SELECT * FROM machine_registry")
        .fetch_all::<IdentRow>()
        .await?;

    let mut registry = BTreeMap::new();
    for IdentRow { vendor, machine, serial, last_active } in rows {
        registry.insert(
            MachineIdentificationUnique {
                vendor,
                machine,
                serial,
            },
            (last_active, false),
        );
    }

    Ok(Arc::new(registry))
}

fn import_schemas(
    registry: Arc<BTreeMap<MachineIdentification, MachineSchema>>,
    incoming: Vec<String>,
) -> anyhow::Result<Arc<BTreeMap<MachineIdentification, MachineSchema>>> {
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

    Ok(Arc::new(new_schemas))
}
