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

// what will be used:
// -> TUI loads the ui in the background and needs to listen for new data
// -> TUI wants to send requests
// -> Rest-Api wants to access current data 
// -> Rest-Api wants to access data in database
// -> Rest-Api wants to submit requests
// -> Database wants to receive 
// -> Controller should not drive/control the tui's cycle or rest-api network stuff

// Want to use database to verify schemas need to hook into the connect process

pub struct ControllerBuilder {
    pub(crate) report_tx: RuntimeReportSender,
    pub(crate) request_tx: RuntimeRequestSender,
    pub(crate) request_rx: RuntimeRequestReceiver,
    pub(crate) machines: Swappable<MachineRegistry>,
    pub(crate) schemas: Swappable<SchemaRegistry>,

    // --- modules ---
    pub(crate) listeners: Vec<Box<dyn Listener>>,
    pub(crate) actors: Vec<RunnerFuture>,
}

impl ControllerBuilder {
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

    pub fn listener<L>(mut self, listener: L) -> Self
    where
        L: Listener + 'static,
    {
        self.listeners.push(Box::new(listener));
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

    pub fn build(self) {
        // TODO: implement
    }
}

impl Default for ControllerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
