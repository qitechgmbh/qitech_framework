use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};
use control_core::RuntimeExport;
use crate::api::RuntimeRequest;

#[derive(Debug)]
pub struct EmbeddedSession {
    tx: broadcast::Sender<Arc<RuntimeExport>>,
    rx: mpsc::Receiver<RuntimeRequest>,
}

impl EmbeddedSession {
    pub(crate) fn new(
        tx: broadcast::Sender<Arc<RuntimeExport>>,
        rx: mpsc::Receiver<RuntimeRequest>,
    ) -> Self {
        Self { rx, tx }
    }

    /// Drains up to `max` currently-buffered requests without blocking.
    pub fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_ {
        std::iter::from_fn(move || self.rx.try_recv().ok()).take(max)
    }

    pub fn export(&mut self, data: RuntimeExport) {
        // ignore errors since a new listener might be addes/re-added
        _ = self.tx.send(Arc::new(data));
    } 
}
