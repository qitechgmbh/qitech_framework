use std::{iter, sync::Arc};
use control_core::{RuntimeReport, RuntimeRequest};
use crate::{RuntimeReportSender, RuntimeRequestReceiver};

#[derive(Debug)]
pub struct EmbeddedSession {
    tx: RuntimeReportSender,
    rx: RuntimeRequestReceiver,
}

impl EmbeddedSession {
    pub(crate) fn new(tx: RuntimeReportSender, rx: RuntimeRequestReceiver) -> Self {
        Self { tx, rx }
    }

    // Drains up to `max` currently-buffered requests without blocking.
    pub fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_ {
        iter::from_fn(move || self.rx.try_recv().ok()).take(max)
    }

    pub fn export(&mut self, data: RuntimeReport) {
        // ignore errors since a new listener might be addes/re-added
        _ = self.tx.send(Arc::new(data));
    } 
}
