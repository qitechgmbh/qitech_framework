use thiserror::Error;

pub trait Transport<In, Out> {
    fn recv(&mut self) -> Result<In, TransportError>;
    fn try_recv(&mut self) -> Result<Option<In>, TransportError>;
    fn send(&mut self, msg: Out) -> Result<(), TransportError>;
}

pub trait AsyncTransport<In, Out> {
    async fn recv(&mut self) -> Result<In, TransportError>;

    fn try_recv(&mut self) -> Result<Option<In>, TransportError>;

    async fn send(&mut self, msg: Out) -> Result<(), TransportError>;
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("connection closed")]
    ConnectionClosed,

    #[error("malformed message: {0}")]
    MalformedMessage(String),

    #[error("peer out of sync")]
    PeerOutOfSync,
}
