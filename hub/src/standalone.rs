use std::sync::Arc;

use anyhow::bail;
use arc_swap::ArcSwap;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use qitech_framework_common::{HandshakeMessage, Hello, MachineSchema, RuntimeReport, RuntimeRequest};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{Config, SchemaRegistry, SharedState};
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
    let (request_tx, mut request_rx) = mpsc::channel(512);

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
        let (conn, _) = listener.accept().await?;
        let mut conn = Framed::new(conn, LengthDelimitedCodec::new());

        if let Err(_) = init_runtime(&mut conn).await {
            // f
            continue;
        }

        // --- connected to client ---
        let (tx, rx) = conn.split();

        let join_res = tokio::join!(
            run_request_dispatcher(&mut request_rx, tx)
        );
    }
}

async fn run_report_receiver(
    tx: &mut broadcast::Sender<Arc<RuntimeReport>>,
    rx: SplitStream<Framed<UnixStream, LengthDelimitedCodec>>,
) {
    loop {
        let report = expect_next::<RuntimeReport>(rx).await.unwrap();
        tx.send(Arc::new(report)).unwrap();
    }
}

async fn run_request_sender(
    rx: &mut mpsc::Receiver<RuntimeRequest>,
    mut tx: SplitSink<Framed<UnixStream, LengthDelimitedCodec>, Bytes>
) {
    loop {
        let request = rx.recv().await.unwrap();
        let bytes = postcard::to_allocvec(&request).unwrap();
        tx.send(Bytes::from(bytes)).await.unwrap();
    }
}

async fn init_runtime(
    conn: &mut Framed<UnixStream, LengthDelimitedCodec>
) -> anyhow::Result<()> {

    // --- receive hello ---
    if Hello::new() != expect_next::<Hello>(conn).await? {
        todo!("Send Rejected");
    }

    // --- handshake phase ---
    let HandshakeMessage::Start = expect_next::<HandshakeMessage>(conn).await? else {
        bail!("Expected Start");
    };

    let mut reg = SchemaRegistry::new();

    loop {
        match expect_next::<HandshakeMessage>(conn).await? {
            HandshakeMessage::RegisterMachine(yaml_str) => {
                let schema = MachineSchema::from_yaml_str(&yaml_str)?;
                
                if reg.insert(schema.identification, schema).is_some() {
                    bail!("Duplicate Schema");
                }
            }

            HandshakeMessage::Finish => {
                break;
            }

            _ => bail!("Unexpected Message")
        }
    }

    // --- handshake complete ---
    Ok(())
}

async fn expect_next<T: DeserializeOwned>(
    conn: &mut Framed<UnixStream, LengthDelimitedCodec>,
) -> anyhow::Result<T> {
    let Some(result) = conn.next().await else {
        bail!("oh no");
    };

    let frame = result?;
    let value: T = postcard::from_bytes(&frame)?;

    Ok(value)
}