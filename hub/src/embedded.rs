use std::sync::Arc;

use arc_swap::ArcSwap;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tracing::info;

use crate::Config;
use crate::IngestManager;
use crate::MachineRegistry;
use crate::RuntimeReportSender;
use crate::RuntimeRequestReceiver;
use crate::SharedState;
use crate::TransactionManager;
use crate::migration;
use crate::utils::import_schemas;
use crate::utils::init_client;
use crate::utils::init_schema_registry;

pub struct Embedded {
    ingest_manager: IngestManager,
    transaction_manager: TransactionManager,
}

macro_rules! measured_operation {
    ($name:literal, $expr:expr) => {{
        let start = std::time::Instant::now();

        tracing::info!(operation = $name, "Started");

        let result = $expr;

        match &result {
            Ok(_) => tracing::info!(
                operation = $name,
                elapsed_ms = start.elapsed().as_millis(),
                "Completed"
            ),
            Err(err) => tracing::error!(
                operation = $name,
                elapsed_ms = start.elapsed().as_millis(),
                error = %err,
                "Failed"
            ),
        }

        result
    }};
}

impl Embedded {
    pub async fn new(
        config: Config,
        schemas: Vec<String>,
    ) -> anyhow::Result<(Self, EmbeddedSession)> {
        use tracing_subscriber::EnvFilter;

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .init();

        tracing::info!("Initialization started");

        let client = measured_operation!(
            "Initialize ClickHouse client",
            Ok::<_, anyhow::Error>(init_client(&config.db))
        )?;

        if config.auto_migrate {
            measured_operation!("Apply migrations", migration::execute(&client).await)?;
        } else {
            measured_operation!("Validate migrations", migration::validate(&client).await)?;
        }

        let client = client.with_database(&config.db.name);

        let runtime_schemas = schemas;

        let schemas = measured_operation!(
            "Initialize schema registry",
            init_schema_registry(&client).await
        )?;

        let machines = measured_operation!(
            "Initialize machine registry",
            MachineRegistry::init(&client, &schemas).await
        )?;

        let schemas = measured_operation!(
            "Import runtime schemas",
            import_schemas(schemas, runtime_schemas)
        )?;

        // --- create channels ---
        let (report_tx, _) = broadcast::channel(64);
        let (pending_tx, pending_rx) = mpsc::channel(512);
        let (request_tx, request_rx) = mpsc::channel(512);

        // --- init state ---
        let state = SharedState {
            config,
            client,
            schemas: Arc::new(ArcSwap::new(Arc::new(schemas))),
            machines: Arc::new(ArcSwap::new(Arc::new(machines))),
            report_tx: report_tx.clone(),
            pending_tx,
        };

        // --- init managers ---
        let ingest_manager = measured_operation!(
            "Initialize ingest manager",
            Ok::<_, anyhow::Error>(IngestManager::init(&state))
        )?;

        let transaction_manager = measured_operation!(
            "Initialize transaction manager",
            TransactionManager::init(&state, pending_rx, request_tx).await
        )?;

        tracing::info!("Initialization completed");

        let hub = Self {
            ingest_manager,
            transaction_manager,
        };

        let session = EmbeddedSession {
            tx: report_tx,
            rx: request_rx,
        };

        Ok((hub, session))
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("Starting Embedded Hub");

        let result = tokio::join!(self.ingest_manager.run(), self.transaction_manager.run(),);

        // TODO: use this maybe?
        _ = result;

        Ok(())
    }
}

#[derive(Debug)]
pub struct EmbeddedSession {
    tx: RuntimeReportSender,
    rx: RuntimeRequestReceiver,
}

impl EmbeddedSession {
    /// Drains up to `max` currently-buffered requests without blocking.
    pub fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_ {
        std::iter::from_fn(move || self.rx.try_recv().ok()).take(max)
    }

    pub fn export(&mut self, data: RuntimeReport) {
        // ignore errors since a new listener might be addes/re-added
        _ = self.tx.send(Arc::new(data));
    }
}
