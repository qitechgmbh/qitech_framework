use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::RuntimeState;
use qitech_framework_common::sync::SendHelloError;
use qitech_framework_common::sync::SubmitStateError;
use qitech_framework_common::sync::SyncRegistryError;
use thiserror::Error;

// --- send hello ---
pub trait BridgeInitializer {
    type Output: Bridge;
    fn send_hello(&mut self) -> Result<(), BridgeInitializeError>;
    fn sync_machine(&mut self, schema: &str) -> Result<(), BridgeInitializeError>;
    fn submit_state(&mut self, state: RuntimeState) -> Result<(), BridgeInitializeError>;
    fn submit_event(&mut self, state: RuntimeInitEvent) -> Result<(), BridgeInitializeError>;
    fn upgrade(self) -> Self::Output;
}

// --- runtime bridge ---
pub trait Bridge {
    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest>;

    /// exports the latest report to the hub
    fn export(&mut self, data: &RuntimeReport);
}

// --- errors ---
#[derive(Error, Debug)]
pub enum BridgeInitializeError {
    #[error("bridge disconnected")]
    BridgeDisconnected,

    #[error("failed to send hello")]
    SendHello(#[from] SendHelloError),

    #[error("failed to sync registry")]
    SyncRegistry(#[from] SyncRegistryError),

    #[error("failed to submit state")]
    SubmitState(#[from] SubmitStateError),
}

// --- mock ---
pub struct MockBridge;

impl BridgeInitializer for MockBridge {
    type Output = MockBridge;

    fn send_hello(&mut self) -> Result<(), BridgeInitializeError> {
        Ok(())
    }

    fn sync_machine(&mut self, schema: &str) -> Result<(), BridgeInitializeError> {
        _ = schema;
        Ok(())
    }

    fn submit_state(&mut self, state: RuntimeState) -> Result<(), BridgeInitializeError> {
        _ = state;
        Ok(())
    }

    fn submit_event(&mut self, state: RuntimeInitEvent) -> Result<(), BridgeInitializeError> {
        _ = state;
        Ok(())
    }

    fn upgrade(self) -> Self::Output {
        self
    }
}

impl Bridge for MockBridge {
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> {
        _ = max;
        std::iter::empty()
    }

    fn export(&mut self, data: &RuntimeReport) {
        _ = data;
    }
}
