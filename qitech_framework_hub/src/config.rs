use std::pin::Pin;

use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::MachineRegistry;
use crate::RuntimeReportSender;
use crate::SchemaRegistry;
use crate::Swappable;
use crate::modules::Actor;
use crate::modules::ActorContext;
use crate::modules::Listener;
use crate::types::RuntimeRequestReceiver;
use crate::types::RuntimeRequestSender;

type RunnerFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct ListenerInstance {
    on_hello_rejected: Pin<Box<dyn Future<Output = ()> + Send>>,
}

pub struct HubConfiguration {
    pub(crate) report_tx: RuntimeReportSender,
    pub(crate) request_tx: RuntimeRequestSender,
    pub(crate) request_rx: RuntimeRequestReceiver,
    pub(crate) machines: Swappable<MachineRegistry>,
    pub(crate) schemas: Swappable<SchemaRegistry>,

    // --- modules ---
    pub(crate) listeners: Vec<Box<dyn Listener>>,
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
            listeners: Default::default(),
        }
    }

    pub fn listener<L: Listener>(mut self, listener: L) -> Self {
        self
    }

    pub fn actor<A: Actor>(mut self, actor: A) -> Self {
        let ctx = ActorContext {
            schemas: self.schemas.clone(),
            machines: self.machines.clone(),
            request_tx: self.request_tx.clone(),
        };

        self.actors.push(Box::pin(actor.run(ctx)));
        self
    }
}

impl Default for HubConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
