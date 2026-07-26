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
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_;

    /// exports the latest report to the hub
    fn export(&mut self, data: &RuntimeReport);
}

// --- errors ---
#[derive(Error, Debug)]
pub enum BridgeInitializeError {
    #[error("data store disconnected")]
    BridgeDisconnected,

    #[error("data store disconnected")]
    SendHello(#[from] SendHelloError),

    #[error("data store disconnected")]
    SyncRegistry(#[from] SyncRegistryError),

    #[error("data store disconnected")]
    SubmitState(#[from] SubmitStateError),
}
