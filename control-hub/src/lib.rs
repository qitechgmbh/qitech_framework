use std::{collections::HashMap, sync::Arc};
use anyhow::anyhow;
use tokio::sync::{mpsc, broadcast, watch};
use arc_swap::{ArcSwap, ArcSwapAny};
use clickhouse::{Client, Row};
use serde::Deserialize;

use control_core::{
    schema,
    MachineIdentification, MachineIdentificationUnique, 
    RuntimeExport, schema::latest::MachineSchema,
};

pub mod vendors {
    include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

    pub fn get(id: u16) -> Option<&'static str> {
        REGISTRY.get(&id).copied()
    }
}

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

mod embedded;
pub use embedded::EmbeddedSession;

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
    pub schemas: Swappable<HashMap<MachineIdentification, MachineSchema>>,

    /// machine registry (ident, connected yes/no)
    pub machines: Swappable<HashMap<MachineIdentificationUnique, bool>>,

    /// channel for forwarding data exports
    pub data_tx: broadcast::Sender<Arc<RuntimeExport>>,

    /// channel for receiving requests
    pub req_tx: mpsc::Sender<RuntimeRequest>,

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
        let client = init_client(&config);
        let registry = init_schema_registry(&client).await?;
        let machines = init_machine_registry(&client).await?;

        // try to import schemas of the runtime
        let schemas = import_schemas(registry, schemas)?;

        let (data_tx, _) = broadcast::channel(64);
        let (req_tx, req_rx) = mpsc::channel(1024);

        let state = SharedState {
            config,
            client,
            schemas: Arc::new(ArcSwap::new(schemas)),
            machines: Arc::new(ArcSwapAny::new(machines)),
            data_tx: data_tx.clone(),
            req_tx,
            shutdown_rx,
        };

        Ok((
            Self { state },
            EmbeddedSession::new(data_tx, req_rx),
        ))
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // simply start all processes
        let result = tokio::join!(
            exporter::run(self.state.clone()),
            // api::run(self.state.clone()),
        );

        // TODO: use this maybe?
        _ = result;

        Ok(())
    }
}

fn init_client(config: &Config) -> Client {
    Client::default()
        .with_url(&config.database.url)
        .with_user(&config.database.user)
        .with_password(&config.database.password)
        .with_database(&config.database.database)
}

/// attempts to load the schema registry from the database
async fn init_schema_registry(
    client: &Client,
) -> anyhow::Result<Arc<HashMap<MachineIdentification, MachineSchema>>> {
    #[derive(Debug, Row, Deserialize)]
    struct SchemaRow {
        content: String,
    }

    let fetched_schemas = client
        .query("SELECT * FROM machine_schemas")
        .fetch_all::<SchemaRow>()
        .await?;

    let mut registry = HashMap::new();

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
) -> anyhow::Result<Arc<HashMap<MachineIdentificationUnique, bool>>> {
    #[derive(Debug, Row, Deserialize)]
    struct IdentRow {
        vendor: u16,
        machine: u16,
        serial: u32,
    }

    let rows = client
        .query("SELECT * FROM registered_machines")
        .fetch_all::<IdentRow>()
        .await?;

    let mut registry = HashMap::new();
    for IdentRow {
        vendor,
        machine,
        serial,
    } in rows
    {
        registry.insert(
            MachineIdentificationUnique {
                vendor,
                machine,
                serial,
            },
            false,
        );
    }

    Ok(Arc::new(registry))
}

fn import_schemas(
    registry: Arc<HashMap<MachineIdentification, MachineSchema>>,
    incoming: Vec<String>,
) -> anyhow::Result<Arc<HashMap<MachineIdentification, MachineSchema>>> {
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
