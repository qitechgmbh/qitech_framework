use std::collections::VecDeque;
use std::println;

use thiserror::Error;

use crate::link::protocol::HandleMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::runtime::session;

pub trait RuntimeTransport: Transport<In = HandleMessage, Out = RuntimeMessage> {}
pub trait HandleTransport: Transport<In = RuntimeMessage, Out = HandleMessage> {}
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

impl RuntimeTransport for MockTransport<HandleMessage, RuntimeMessage> {}
impl HandleTransport for MockTransport<RuntimeMessage, HandleMessage> {}

// --- runtime simple ---
pub struct DebugRuntimeTransport {
    state: u8,
}

impl DebugRuntimeTransport {
    pub fn start_session() -> session::SendHello<Self> {
        session::SendHello::new(Self { state: 0 })
    }
}

impl Transport for DebugRuntimeTransport {
    type In = HandleMessage;
    type Out = RuntimeMessage;

    fn recv(&mut self) -> Result<Self::In, TransportError> {
        match self.state {
            0 => {
                self.state = 1;
                Ok(HandleMessage::HelloAck)
            }
            1 => {
                self.state = 1;
                Ok(HandleMessage::SchemaAck)
            }
            _ => unreachable!(),
        }
    }

    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError> {
        Ok(None)
    }

    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError> {
        match msg {
            RuntimeMessage::Hello(hello) => println!("{hello:#?}"),
            RuntimeMessage::Schema(schema) => {
                println!("sending schema for: {:#?}", schema.identification)
            }
            RuntimeMessage::InitEvent(event) => println!("{event:#?}"),
            RuntimeMessage::Finished => println!("finished"),
            RuntimeMessage::Report(_) => {
                println!("sending report");
            }
        }

        // println!("Sending: {msg:#?}");
        _ = msg;
        Ok(())
    }
}

impl RuntimeTransport for DebugRuntimeTransport {}
