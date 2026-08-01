use std::collections::VecDeque;
use thiserror::Error;

use crate::session::protocol::HandleMessage;
use crate::session::protocol::RuntimeMessage;

pub trait AgentTransport: Transport<In = HandleMessage, Out = RuntimeMessage> {}
pub trait ControllerTransport: Transport<In = RuntimeMessage, Out = HandleMessage> {}
pub trait AsyncHandleTransport: AsyncTransport<In = RuntimeMessage, Out = HandleMessage> {}

pub trait Transport {
    type In;
    type Out;

    fn recv(&mut self) -> Result<Self::In, TransportError>;
    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError>;
    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError>;
}

pub trait AsyncTransport {
    type In;
    type Out;

    fn recv(&mut self) -> impl Future<Output = Result<Self::In, TransportError>> + Send;
    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError>;
    fn send(&mut self, msg: Self::Out) -> impl Future<Output = Result<(), TransportError>> + Send;
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("connection closed")]
    Disconnected,

    #[error("malformed message: {0}")]
    MalformedMessage(String),

    #[error("peer out of sync")]
    PeerOutOfSync,
}

// --- mock ---
pub type MockRuntimeTransport = MockTransport<RuntimeMessage, HandleMessage>;

pub type MockHandleTransport = MockTransport<HandleMessage, RuntimeMessage>;

pub struct MockTransport<I, O> {
    incoming: VecDeque<I>,
    outgoing: VecDeque<O>,
}

impl<I, O> MockTransport<I, O> {
    pub fn new() -> Self {
        Self {
            incoming: VecDeque::new(),
            outgoing: VecDeque::new(),
        }
    }

    pub fn with_messages(messages: impl IntoIterator<Item = I>) -> Self {
        Self {
            incoming: messages.into_iter().collect(),
            outgoing: VecDeque::new(),
        }
    }

    pub fn push_incoming(&mut self, msg: I) {
        self.incoming.push_back(msg);
    }

    pub fn take_outgoing(&mut self) -> Vec<O> {
        self.outgoing.drain(..).collect()
    }
}

impl<I, O> Default for MockTransport<I, O> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> Transport for MockTransport<I, O> {
    type In = I;
    type Out = O;

    fn recv(&mut self) -> Result<Self::In, TransportError> {
        self.incoming
            .pop_front()
            .ok_or(TransportError::Disconnected)
    }

    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError> {
        Ok(self.incoming.pop_front())
    }

    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError> {
        self.outgoing.push_back(msg);
        Ok(())
    }
}

impl AgentTransport for MockTransport<HandleMessage, RuntimeMessage> {}
impl ControllerTransport for MockTransport<RuntimeMessage, HandleMessage> {}
