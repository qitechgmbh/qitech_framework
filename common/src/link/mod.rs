use crate::RuntimeReport;
use crate::RuntimeRequest;

mod crossbeam;
pub mod error;
mod handle;
mod protocol;
mod runtime;

mod transport;
use transport::AsyncTransport;
use transport::Transport;

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
