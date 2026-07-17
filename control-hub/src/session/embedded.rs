use std::pin::Pin;

use control_core::{RuntimeReport, RuntimeRequest};
use tokio::sync::mpsc;

use super::{Listener, PendingConnection, SessionTransport};

pub struct EmbeddedListener {
    connection: Option<EmbeddedPendingConnection>,
}

impl Listener for EmbeddedListener {
    type Pending = EmbeddedPendingConnection;
    type Transport = EmbeddedTransport;

    async fn accept(&mut self) -> anyhow::Result<Self::Pending> {
        self.connection
            .take()
            .ok_or_else(|| anyhow::anyhow!("embedded runtime already connected"))
    }

    async fn upgrade(
        &self,
        conn: Self::Pending,
    ) -> anyhow::Result<Self::Transport> {
        Ok(EmbeddedTransport {
            request_tx: conn.request_tx,
            report_rx: conn.report_rx,
        })
    }
}

pub struct EmbeddedPendingConnection {
    request_tx: mpsc::Sender<RuntimeRequest>,
    report_rx: mpsc::Receiver<RuntimeReport>,
}

impl PendingConnection for EmbeddedPendingConnection {
    async fn recv(&mut self) -> anyhow::Result<Vec<String>> {
        Ok(std::mem::take(&mut self.schemas))
    }
}

pub struct EmbeddedTransport {
    request_tx: mpsc::Sender<RuntimeRequest>,
    report_rx: mpsc::Receiver<RuntimeReport>,
}

impl SessionTransport for EmbeddedTransport {
    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<RuntimeReport>> + Send + '_>> {
        Box::pin(async move {
            self.report_rx
                .recv()
                .await
                .ok_or_else(|| anyhow::anyhow!("runtime stopped"))
        })
    }

    fn send(
        &mut self,
        request: RuntimeRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.request_tx.send(request).await?;
            Ok(())
        })
    }
}
