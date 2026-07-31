use crate::MachineSchema;
use crate::RuntimeInitEvent;
use crate::RuntimeReport;
use crate::RuntimeRequest;
use crate::link::error::HandshakeError;
use crate::link::protocol::HandleMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::transport::HandleTransport;

// --- receive hello ---
pub struct ReceiveHello<T> {
    transport: T,
}

impl<T> ReceiveHello<T>
where
    T: HandleTransport,
{
    pub(crate) fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn complete(mut self) -> Result<SyncSchemas<T>, HandshakeError> {
        match self.transport.recv()? {
            RuntimeMessage::Hello(_) => {
                self.transport.send(HandleMessage::HelloAck)?;

                Ok(SyncSchemas {
                    transport: self.transport,
                })
            }

            other => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
        }
    }
}

// --- sync schemas ---
pub struct SyncSchemas<T> {
    transport: T,
}

impl<T> SyncSchemas<T>
where
    T: HandleTransport,
{
    pub fn sync<F>(mut self, mut handler: F) -> Result<Initializing<T>, HandshakeError>
    where
        F: FnMut(MachineSchema) -> Result<(), String>,
    {
        loop {
            match self.transport.recv()? {
                RuntimeMessage::Schema(schema) => match handler(*schema) {
                    Ok(()) => {
                        self.transport.send(HandleMessage::SchemaAck)?;
                    }

                    Err(reason) => {
                        self.transport
                            .send(HandleMessage::SchemaReject(reason.clone()))?;

                        return Err(HandshakeError::SchemaRejected(reason));
                    }
                },

                RuntimeMessage::Finished => {
                    return Ok(Initializing {
                        transport: self.transport,
                    });
                }

                other => {
                    return Err(HandshakeError::UnexpectedMessage(format!("{other:?}")));
                }
            }
        }
    }
}

// --- initializing ---
pub struct Initializing<T> {
    transport: T,
}

impl<T> Initializing<T>
where
    T: HandleTransport,
{
    pub fn complete<F>(mut self, mut handler: F) -> Result<Running<T>, HandshakeError>
    where
        F: FnMut(RuntimeInitEvent) -> Result<(), String>,
    {
        loop {
            match self.transport.recv()? {
                RuntimeMessage::InitEvent(event) => {
                    if let Err(reason) = handler(event) {
                        return Err(HandshakeError::InitializationFailed(reason));
                    }
                }

                RuntimeMessage::Finished => {
                    return Ok(Running {
                        transport: self.transport,
                    });
                }

                other => {
                    return Err(HandshakeError::UnexpectedMessage(format!("{other:?}")));
                }
            }
        }
    }
}

// --- running ---
pub struct Running<T> {
    transport: T,
}

impl<T> Running<T>
where
    T: HandleTransport,
{
    /// non blocking send
    pub fn send_request(&mut self, request: RuntimeRequest) -> Result<(), HandshakeError> {
        self.transport.send(HandleMessage::Request(request))?;
        Ok(())
    }

    /// blocking read
    pub fn recv_report(&mut self) -> Result<RuntimeReport, HandshakeError> {
        match self.transport.recv()? {
            RuntimeMessage::Report(report) => Ok(*report),
            other => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
        }
    }
}
