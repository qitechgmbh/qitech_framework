use std::println;

use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;

use crate::runtime::error::BridgeBootstrapError;

pub mod crossbeam;
pub use crossbeam::Handle as CrossbeamHandle;
pub use crossbeam::HelloHandle as CrossbeamHelloHandle;
pub use crossbeam::InitHandle as CrossbeamInitHandle;
pub use crossbeam::RuntimeInitEvent as CrossbeamRuntimeInitEvent;

pub trait RuntimeHandle: Sized {
    type Handshake: RuntimeHandleHandshake<Self>;

    fn submit_request(&mut self, request: RuntimeRequest);
    fn recv_report(&mut self) -> Option<RuntimeReport>;
}

pub trait RuntimeHandleHandshake<H: RuntimeHandle> {
    fn send_hello(&mut self) -> Result<(), RuntimeHandleHandshakeError>;

    fn recv_schema(&mut self, schema: &str) -> Result<(), RuntimeHandleHandshakeError>;

    fn recv_event(&mut self, event: RuntimeInitEvent) -> Result<(), RuntimeHandleHandshakeError>;

    fn complete(self) -> Result<H, RuntimeHandleHandshakeError>;
}

pub trait RuntimeSessionHandshake<B: RuntimeSession> {
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

    fn complete(self) -> Result<B, BridgeBootstrapError>;
}

pub trait RuntimeSession: Sized {
    type Handshake: RuntimeSessionHandshake<Self>;

    /// Retrieves the next request from the bridge buffer if any
    fn get_request(&mut self) -> Option<RuntimeRequest>;

    /// exports the latest report over the bridge
    fn export(&mut self, data: &RuntimeReport);
}

// --- mock ---
pub struct MockSession;

impl RuntimeSessionHandshake<MockSession> for MockSession {
    type FinishedPayload = ();
    fn complete(self) -> Result<MockSession, BridgeBootstrapError> {
        Ok(self)
    }

    fn submit_event(
        &mut self,
        state: RuntimeInitEvent<Self::FinishedPayload>,
    ) -> Result<(), BridgeBootstrapError> {
        println!("sending event: {state:#?}");
        Ok(())
    }
}

impl RuntimeSession for MockSession {
    type Handshake = MockSession;

    fn get_request(&mut self) -> Option<RuntimeRequest> {
        None
    }

    fn export(&mut self, data: &RuntimeReport) {
        _ = data;
    }
}
