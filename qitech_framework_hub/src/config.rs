use std::pin::Pin;

use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::MachineRegistry;
use crate::RuntimeReportSender;
use crate::SchemaRegistry;
use crate::Swappable;
use crate::modules::Runner;
use crate::modules::RunnerContext;
use crate::types::RuntimeRequestReceiver;
use crate::types::RuntimeRequestSender;

type RunnerFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync>>;

pub struct HubConfiguration {
    pub(crate) report_tx: RuntimeReportSender,
    pub(crate) request_tx: RuntimeRequestSender,
    pub(crate) request_rx: RuntimeRequestReceiver,
    pub(crate) machines: Swappable<MachineRegistry>,
    pub(crate) schemas: Swappable<SchemaRegistry>,
    pub(crate) actors: Vec<RunnerFuture>,
}

impl HubConfiguration {
    pub fn new() -> Self {
        let (report_tx, _) = broadcast::channel(32);
        let (request_tx, request_rx) = mpsc::channel(128);

        Self {
            report_tx,
            request_tx,
            request_rx,
            machines: Default::default(),
            schemas: Default::default(),
            actors: Default::default(),
        }
    }

    pub fn reactor(mut self) -> Self {
        self
    }

    pub fn runner<R: Runner>(mut self, runner: R) -> Self {
        let ctx = RunnerContext {
            schemas: self.schemas.clone(),
            machines: self.machines.clone(),
            request_tx: self.request_tx.clone(),
        };

        // self.runners.push(Box::pin(runner.run(ctx)));
        self
    }
}

impl Default for HubConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
