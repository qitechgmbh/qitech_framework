use std::sync::Arc;
use control_core::{RuntimeReport, RuntimeRequest};
use crate::{RuntimeReportSender, RuntimeRequestReceiver};

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
