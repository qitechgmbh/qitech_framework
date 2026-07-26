use std::sync::Arc;

use arc_swap::ArcSwap;
use tokio::net::UnixListener;
use tokio::sync::{broadcast, mpsc};

use crate::{Config, SharedState};
use crate::ingest::IngestManager;
use crate::migration;
use crate::transaction::TransactionManager;
use crate::utils::init_client;

pub struct Server {
    ingest_manager: IngestManager,
    transaction_manager: TransactionManager,
}

pub async fn run(listener: UnixListener, config: Config) -> anyhow::Result<()> {
    let client = init_client(&config.db);

    if config.auto_migrate {
        migration::execute(&client).await?;
    } else {
        migration::validate(&client).await?;
    }

    // migration is complete and database accessible. Upgrade to database
    let client = client.with_database(&config.db.name);

    // --- create channels ---
    let (report_tx, _) = broadcast::channel(64);
    let (pending_tx, pending_rx) = mpsc::channel(512);
    let (request_tx, request_rx) = mpsc::channel(512);

    // --- init state ---
    let state = SharedState {
        config,
        client,
        schemas: Arc::new(ArcSwap::new(Arc::new(Default::default()))),
        machines: Arc::new(ArcSwap::new(Arc::new(Default::default()))),
        report_tx: report_tx.clone(),
        pending_tx,
    };

    // --- init managers ---
    let ingest_manager = IngestManager::init(&state);
    let transaction_manager = TransactionManager::init(&state, pending_rx, request_tx).await;

    loop {
        let (conn, addr) = listener.accept().await?;

        
    }
}
