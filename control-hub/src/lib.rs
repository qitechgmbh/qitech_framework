use std::collections::BTreeMap;
use std::sync::Arc;
use anyhow::anyhow;
use control_core::{RuntimeRequest, RuntimeRequestKind};
use serde::Deserialize;
use arc_swap::ArcSwap;
use tokio::spawn;
use tokio::sync::oneshot;
use tokio::sync::{mpsc, broadcast, watch};
use clickhouse::{Client, Row};

use control_core::{
    schema,
    MachineIdentification, 
    RuntimeReport, schema::latest::MachineSchema,
};

mod migration;
pub use migration::migrate;

mod session;

mod config;
pub use config::Config;
pub use config::DatabaseConfig;

mod embedded;
pub use embedded::EmbeddedSession;

mod tables;

mod runtime_ingest;

mod machine_registry;
use machine_registry::MachineRegistry;

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

    pub report_tx: RuntimeReportSender,

    /// channel to start a transaction
    pub request_tx: mpsc::Sender<RuntimeRequestKind>,

    pub shutdown_rx: watch::Receiver<()>,

    // TODO: implemen
    // pub runtime_state: RuntimeState
}

pub struct ControlHub {
    state: SharedState,
}

// Session Manager -> tries to connect, holds the request rx and report tx

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

        let state = SharedState {
            config,
            client,
            schemas: Arc::new(ArcSwap::new(Arc::new(schemas))),
            machines: Arc::new(ArcSwap::new(Arc::new(machines))),
            report_tx: data_tx.clone(),
            request_tx: req_tx,
            shutdown_rx,
        };

        Ok((
            Self { state },
            EmbeddedSession::new(data_tx, req_rx),
        ))
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let x = self.state.clone();

        let result = tokio::join!(
            // takes in the data, updates cache and handles persistance in the db
            spawn(async move {
                let r = runtime_ingest::run(x).await;
                _ = r;
                //println!("EXITING: {r:?}");
            }),

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
        .with_setting("allow_experimental_json_type", "1")
        // Enable inserting JSON columns as a string
        .with_setting("input_format_binary_read_json_as_string", "1")
        ;

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
