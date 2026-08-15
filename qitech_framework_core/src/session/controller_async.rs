use crate::report::RuntimeInitEvent;
use crate::report::RuntimeReport;
use crate::request::RuntimeRequest;
use crate::schema::MachineSchema;
use crate::session::error::HandshakeError;
use crate::session::error::SchemaSyncError;
use crate::session::error::SessionRecvError;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::transport::AsyncControllerTransport;
use crate::session::transport::TransportError;

// --- acknowledge hello ---
pub struct SessionHandshake<T> {
    transport: T,
}

impl<T> SessionHandshake<T>
where
    T: AsyncControllerTransport,
{
    pub(crate) fn new(transport: T) -> Self {
        Self { transport }
    }

    pub async fn complete(mut self) -> Result<SessionSyncingSchemas<T>, HandshakeError> {
        match self.transport.recv().await? {
            RuntimeMessage::Hello(_) => {
                self.transport.send(ControllerMessage::HelloAck).await?;

                Ok(SessionSyncingSchemas {
                    transport: self.transport,
                })
            }

            other => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
        }
    }
}

// --- sync schemas ---
pub struct SessionSyncingSchemas<T> {
    transport: T,
}

impl<T> SessionSyncingSchemas<T>
where
    T: AsyncControllerTransport,
{
    pub async fn sync<F>(mut self, mut handler: F) -> Result<SessionInitializing<T>, HandshakeError>
    where
        F: FnMut(MachineSchema) -> Result<(), SchemaSyncError>,
    {
        loop {
            match self.transport.recv().await? {
                RuntimeMessage::Schema(schema) => match handler(*schema) {
                    Ok(()) => {
                        self.transport.send(ControllerMessage::SchemaAck).await?;
                    }

                    Err(reason) => {
                        self.transport
                            .send(ControllerMessage::SchemaReject(reason.clone()))
                            .await?;
                        return Err(HandshakeError::SchemaRejected(reason));
                    }
                },

                RuntimeMessage::Finished => {
                    return Ok(SessionInitializing {
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

// --- receive init events ---
pub struct SessionInitializing<T> {
    transport: T,
}

impl<T> SessionInitializing<T>
where
    T: AsyncControllerTransport,
{
    pub async fn complete<F>(mut self, mut handler: F) -> Result<SessionRunning<T>, HandshakeError>
    where
        F: FnMut(RuntimeInitEvent) -> Result<(), String>,
    {
        loop {
            match self.transport.recv().await? {
                RuntimeMessage::InitEvent(event) => {
                    if let Err(reason) = handler(event) {
                        return Err(HandshakeError::InitializationFailed(reason));
                    }
                }

                RuntimeMessage::Finished => {
                    return Ok(SessionRunning {
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

// --- run ---
pub struct SessionRunning<T> {
    transport: T,
}

impl<T> SessionRunning<T>
where
    T: AsyncControllerTransport,
{
    pub async fn recv_report(&mut self) -> Result<RuntimeReport, SessionRecvError> {
        match self.transport.recv().await? {
            RuntimeMessage::Report(report) => Ok(*report),
            other => Err(SessionRecvError::UnexpectedMessage(format!("{other:?}"))),
        }
    }

    pub async fn send_request(&mut self, request: RuntimeRequest) -> Result<(), TransportError> {
        self.transport
            .send(ControllerMessage::Request(request))
            .await?;

        Ok(())
    }
}
