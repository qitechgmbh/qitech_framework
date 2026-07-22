use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use anyhow::bail;
use tokio::{select, sync::{broadcast, mpsc, oneshot}};
use control_core::{OperationResult, RuntimeReport, RuntimeRequest, RuntimeRequestKind};
use crate::{RuntimeReportReceiver, SharedState};

pub type TransactionId = u64;

pub struct PendingRuntimeRequest {
    address: SocketAddr,
    request: RuntimeRequestKind, 
    response_tx: oneshot::Sender<OperationResult>,
}

pub struct TransactionManager {
    /// current id count used for creating a transaction id.
    /// Incremented after each request.
    id_counter: u64,

    /// channel for receiving runtime reports
    report_rx: RuntimeReportReceiver,

    /// pending requests coming from the api
    pending_rx: mpsc::Receiver<PendingRuntimeRequest>,

    /// channel to forward signed request to session/runtime
    request_tx: mpsc::Sender<RuntimeRequest>,

    /// transaction registry of pending transactions
    transactions: HashMap<TransactionId, oneshot::Sender<OperationResult>>,
}

impl TransactionManager {
    pub async fn init(
        state: &SharedState,
        pending_rx: mpsc::Receiver<PendingRuntimeRequest>,
        request_tx: mpsc::Sender<RuntimeRequest>,
    ) -> anyhow::Result<Self> {
        let sql = "SELECT max(transaction_id) FROM runtime_transactions";
        let id_counter = state.client.query(sql).fetch_one::<u64>().await?;

        Ok(Self { 
            id_counter, 
            report_rx: state.report_tx.subscribe(), 
            pending_rx, 
            request_tx, 
            transactions: HashMap::new(), 
        })
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            select! {
                biased;
                res = self.report_rx.recv() => {
                    use broadcast::error::RecvError;

                    let report = match res {
                        Ok(v) => v,
                        Err(RecvError::Closed) => {
                            println!("[TransactionManager] report dispatcher closed, exiting ...");
                            continue;
                        },
                        Err(RecvError::Lagged(count)) => {
                            eprintln!("[TransactionManager] Lagged behind and missed {count} reports");
                            bail!("Critical state: Out of sync with system");
                        }
                    };

                    self.process_report(report);
                }

                opt = self.pending_rx.recv() => {
                    let Some(transaction) = opt else {
                        println!("[TransactionManager] No more senders, exiting ...");
                        return Ok(());
                    };

                    let transaction_id = self.id_counter;
                    self.id_counter += 1;

                    // put into registry
                    self.transactions.insert(transaction_id, transaction.response_tx);

                    let request = RuntimeRequest { 
                        transaction_id, 
                        kind: transaction.request 
                    };

                    use mpsc::error::TrySendError;
                    match self.request_tx.try_send(request) {
                        Ok(_) => {},
                        Err(TrySendError::Full(_)) => {
                            // remove entry again since we won't process it
                            let response_tx = self.transactions.remove(&transaction_id)
                                .expect("must exist");

                            _ = response_tx;
                            // response_tx.send();

                            todo!("Send rejected because too many requests")
                        },
                        Err(TrySendError::Closed(_)) => {
                            todo!("Send rejected because server shutting down, then exit.")
                        }
                    };
                }
            }
        }
    }

    fn process_report(&mut self, report: Arc<RuntimeReport>) {
        for (id, result) in &report.responses {
            let Some(response_tx) = self.transactions.remove(id) else {
                eprintln!("Received response for unregistered transaction {id}");
                continue;
            };

            if response_tx.send(*result).is_err() {
                eprintln!("Failed to dispatch response, channel closed");
            };
        }
    }
}
