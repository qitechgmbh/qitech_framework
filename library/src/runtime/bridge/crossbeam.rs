use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::{self};
use qitech_framework_common::Hello;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::sync::HelloAck;

use crate::runtime::bridge::Bridge;
use crate::runtime::bridge::BridgeBootstrap;
use crate::runtime::error::BridgeBootstrapError;

// --- payload ---
type HelloAckPayload = HelloAck<Sender<RuntimeInitEvent>>;
pub type RuntimeInitEvent = qitech_framework_common::RuntimeInitEvent<Handle>;

// --- handle ---
#[derive(Debug)]
pub struct HelloHandle {
    hello_rx: Receiver<Hello>,
    ack_tx: Sender<HelloAckPayload>,
}

impl HelloHandle {
    pub fn handle_hello(self) -> Result<InitHandle, HelloHandle> {
        let hello = self.hello_rx.recv().unwrap();

        if hello != Hello::new() {
            self.ack_tx.send(HelloAck::Rejected).unwrap();
            return Err(self);
        }

        let (event_tx, event_rx) = channel::unbounded();
        self.ack_tx.send(HelloAck::Accepted(event_tx)).unwrap();

        Ok(InitHandle { event_rx })
    }
}

#[derive(Debug)]
pub struct InitHandle {
    event_rx: Receiver<RuntimeInitEvent>,
}

impl InitHandle {
    pub fn recv(&mut self) -> RuntimeInitEvent {
        self.event_rx.recv().unwrap()
    }
}

#[derive(Debug)]
pub struct Handle {
    request_tx: Sender<RuntimeRequest>,
    report_rx: Receiver<RuntimeReport>,
}

impl Handle {
    pub fn recv(&mut self) -> Option<RuntimeReport> {
        self.report_rx.try_recv().ok()
    }

    pub fn send(&mut self, request: RuntimeRequest) {
        self.request_tx.send(request).unwrap()
    }
}

// --- bridge ---
pub enum CrossbeamBridgeBootstrap {
    Hello {
        hello_tx: Sender<Hello>,
        ack_rx: Receiver<HelloAckPayload>,
    },
    Initialize {
        event_tx: Sender<RuntimeInitEvent>,
    },
    Running {
        request_rx: Receiver<RuntimeRequest>,
        report_tx: Sender<RuntimeReport>,
    },
}

impl CrossbeamBridgeBootstrap {
    pub fn new() -> (Self, HelloHandle) {
        let (hello_tx, hello_rx) = crossbeam::channel::unbounded();
        let (ack_tx, ack_rx) = crossbeam::channel::unbounded();

        let bridge = Self::Hello { hello_tx, ack_rx };
        let handle = HelloHandle { hello_rx, ack_tx };

        (bridge, handle)
    }
}

impl BridgeBootstrap<CrossbeamBridge> for CrossbeamBridgeBootstrap {
    type FinishedPayload = Handle;

    fn send_hello(&mut self) -> Result<(), BridgeBootstrapError> {
        let CrossbeamBridgeBootstrap::Hello { hello_tx, ack_rx } = self else {
            panic!("Not in hello state anymore");
        };

        hello_tx.send(Hello::new()).unwrap();

        match ack_rx.recv().unwrap() {
            HelloAck::Accepted(event_tx) => {
                *self = CrossbeamBridgeBootstrap::Initialize { event_tx };
                Ok(())
            }
            HelloAck::Rejected => panic!("not accepted sadge"),
        }
    }

    fn submit_event(&mut self, event: RuntimeInitEvent) -> Result<(), BridgeBootstrapError> {
        let CrossbeamBridgeBootstrap::Initialize { event_tx } = self else {
            panic!("Not in initialize state");
        };

        event_tx.send(event).unwrap();
        Ok(())
    }

    fn finish(self) -> Result<CrossbeamBridge, BridgeBootstrapError> {
        let CrossbeamBridgeBootstrap::Initialize { event_tx } = self else {
            panic!("Not in initialize state");
        };

        let (request_tx, request_rx) = channel::unbounded();
        let (report_tx, report_rx) = channel::unbounded();

        let handle = Handle {
            request_tx,
            report_rx,
        };

        event_tx.send(RuntimeInitEvent::Finished(handle)).unwrap();
        Ok(CrossbeamBridge {
            request_rx,
            report_tx,
        })
    }
}

// --- bridge ---
pub struct CrossbeamBridge {
    request_rx: Receiver<RuntimeRequest>,
    report_tx: Sender<RuntimeReport>,
}

impl Bridge for CrossbeamBridge {
    type Bootstrap = CrossbeamBridgeBootstrap;

    fn get_request(&mut self) -> Option<RuntimeRequest> {
        match self.request_rx.try_recv() {
            Ok(v) => Some(v),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("Nouuuuuuu"),
        }
    }

    fn export(&mut self, data: &RuntimeReport) {
        let report = data.clone();
        self.report_tx.send(report).unwrap();
    }
}
