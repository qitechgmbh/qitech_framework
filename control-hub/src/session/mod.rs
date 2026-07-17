use std::{pin::Pin, sync::Arc};
use control_core::{RuntimeReport, RuntimeRequest};
use crate::{RuntimeReportSender, RuntimeRequestReceiver, SharedState};

mod embedded;
mod unix_socket;

pub trait Session {
    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_;

    /// dispatch data to hub
    fn export(&mut self, data: RuntimeReport);
}

#[derive(Debug)]
pub struct EmbeddedSession {
    tx: RuntimeReportSender,
    rx: RuntimeRequestReceiver,
}

impl EmbeddedSession {
    pub(crate) fn new(tx: RuntimeReportSender, rx: RuntimeRequestReceiver) -> Self {
        Self { rx, tx }
    }
}

impl Session for EmbeddedSession {
    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_ {
        std::iter::from_fn(move || self.rx.try_recv().ok()).take(max)
    }

    fn export(&mut self, data: RuntimeReport) {
        // ignore errors since a new listener might be addes/re-added
        _ = self.tx.send(Arc::new(data));
    } 
}

/// manages the session lifecycle with the runtime
pub struct SessionManager<L>
where
    L: Listener,
{
    state: SharedState,

    request_rx: RuntimeRequestReceiver,
    report_tx: RuntimeReportSender,

    listener: L,
    session: Option<L::Transport>,
}

impl<L> SessionManager<L>
where
    L: Listener,
{
    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let pending = self.listener.accept().await?;

            let transport = match self.handshake(pending).await {
                Ok(session) => session,
                Err(err) => {
                    eprintln!("Handshake failed: {err}");
                    continue;
                }
            };

            self.session = Some(transport);
            self.run_session().await?;
            self.session = None;
        }
    }

    async fn handshake(
        &self,
        mut pending: L::Pending,
    ) -> anyhow::Result<L::Transport> {
        let schemas = pending.recv().await?;

        // validate_schemas(&schemas)?;
        // TODO: implement

        self.listener.upgrade(pending).await
    }

    async fn run_session(&mut self) -> anyhow::Result<()> {
        let session = self.session.as_mut().unwrap();

        loop {
            tokio::select! {
                request = self.request_rx.recv() => {
                    let Some(request) = request else {
                        return Ok(());
                    };

                    session.send(request).await?;
                }

                report = session.recv() => {
                    let report = report?;
                    let _ = self.report_tx.send(Arc::new(report));
                }
            }
        }
    }
}

pub trait Listener {
    type Pending: PendingConnection;
    type Transport: SessionTransport;

    async fn accept(&mut self) -> anyhow::Result<Self::Pending>;
    async fn upgrade(&self, conn: Self::Pending) -> anyhow::Result<Self::Transport>;
}

pub trait PendingConnection {
    async fn recv(&mut self) -> anyhow::Result<Vec<String>>;
}

pub trait SessionTransport: Send {
    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<RuntimeReport>> + Send + '_>>;

    fn send(
        &mut self,
        request: RuntimeRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>>;
}
