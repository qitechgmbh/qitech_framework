use std::sync::mpsc;

use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;

use crate::runtime::error::BridgeBootstrapError;

pub trait BridgeBootstrap<T: Bridge> {
    fn send_hello(&mut self) -> Result<(), BridgeBootstrapError> {
        Ok(())
    }

    fn sync_machine(&mut self, schema: &str) -> Result<(), BridgeBootstrapError> {
        _ = schema;
        Ok(())
    }

    fn submit_event(&mut self, state: RuntimeInitEvent) -> Result<(), BridgeBootstrapError> {
        _ = state;
        Ok(())
    }

    fn finish(self) -> T;
}

pub trait Bridge: Sized {
    type Bootstrap: BridgeBootstrap<Self>;

    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest>;

    /// exports the latest report to the hub
    fn export(&mut self, data: &RuntimeReport);
}

// --- mock ---
pub struct MockBridge;

impl BridgeBootstrap<MockBridge> for MockBridge {
    fn finish(self) -> MockBridge { self }
}

impl Bridge for MockBridge {
    type Bootstrap = MockBridge;

    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> {
        _ = max;
        std::iter::empty()
    }

    fn export(&mut self, data: &RuntimeReport) {
        _ = data;
    }
}

// --- debug bridge ---
pub struct DebugBridge {
    pub rx: mpsc::Receiver<RuntimeRequest>,
}

impl BridgeBootstrap<DebugBridge> for DebugBridge {
    fn finish(self) -> DebugBridge { self }
}

impl Bridge for DebugBridge {
    type Bootstrap = DebugBridge;

    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> {
        std::iter::from_fn(|| self.rx.try_recv().ok())
            .take(max)
    }

    fn export(&mut self, data: &RuntimeReport) {
        println!("{data:#?}");
    }
}

// --- tokio mpsc ---

// --- unix socket ---
