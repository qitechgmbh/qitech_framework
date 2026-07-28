use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;

use crate::runtime::error::BridgeBootstrapError;

pub mod crossbeam;
pub use crossbeam::HelloHandle as CrossbeamHelloHandle;
pub use crossbeam::InitHandle as CrossbeamInitHandle;
pub use crossbeam::Handle as CrossbeamHandle;
pub use crossbeam::RuntimeInitEvent as CrossbeamRuntimeInitEvent;

pub trait BridgeBootstrap<B: Bridge> {
    type FinishedPayload;

    fn send_hello(&mut self) -> Result<(), BridgeBootstrapError> {
        Ok(())
    }

    fn sync_machine(&mut self, schema: &str) -> Result<(), BridgeBootstrapError> {
        _ = schema;
        Ok(())
    }

    fn submit_event(
        &mut self, 
        state: RuntimeInitEvent<Self::FinishedPayload>,
    ) -> Result<(), BridgeBootstrapError> {
        _ = state;
        Ok(())
    }

    fn finish(self) -> Result<B, BridgeBootstrapError>;
}

pub trait Bridge: Sized {
    type Bootstrap: BridgeBootstrap<Self>;

    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_request(&mut self) -> Option<RuntimeRequest>;

    /// exports the latest report to the hub
    fn export(&mut self, data: &RuntimeReport);
}

// --- mock ---
pub struct MockBridge;

impl BridgeBootstrap<MockBridge> for MockBridge {
    type FinishedPayload = ();
    fn finish(self) -> Result<MockBridge, BridgeBootstrapError> { Ok(self) }
}

impl Bridge for MockBridge {
    type Bootstrap = MockBridge;

    fn get_request(&mut self) -> Option<RuntimeRequest> {
        None
    }

    fn export(&mut self, data: &RuntimeReport) {
        _ = data;
    }
}
