use crate::MachineSchema;
use crate::RuntimeInitEvent;
use crate::RuntimeReport;
use crate::RuntimeRequest;
use crate::session::error::HandshakeError;
use crate::session::protocol::HandleMessage;
use crate::session::protocol::Hello;
use crate::session::protocol::RuntimeMessage;
use crate::session::transport::AgentTransport;

// --- send hello ---
pub struct SendHello<T>
where
    T: AgentTransport,
{
    transport: T,
}

impl<T> SendHello<T>
where
    T: AgentTransport,
{
    pub(crate) fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T> SendHello<T>
where
    T: AgentTransport,
{
    pub fn complete(mut self) -> Result<SyncSchemas<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Hello(Hello::new()))?;

        match self.transport.recv()? {
            HandleMessage::HelloAck => Ok(SyncSchemas {
                transport: self.transport,
            }),

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
    T: AgentTransport,
{
    pub fn sync_schema(&mut self, schema: MachineSchema) -> Result<(), HandshakeError> {
        self.transport
            .send(RuntimeMessage::Schema(Box::new(schema)))?;

        match self.transport.recv()? {
            HandleMessage::SchemaAck => Ok(()),

            HandleMessage::SchemaReject(reason) => Err(HandshakeError::SchemaRejected(reason)),

            other => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
        }
    }

    pub fn complete(mut self) -> Result<Initializing<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Finished)?;

        Ok(Initializing {
            transport: self.transport,
        })
    }
}

// --- initializing ---
pub struct Initializing<T> {
    transport: T,
}

impl<T> Initializing<T>
where
    T: AgentTransport,
{
    pub fn send_event(&mut self, event: RuntimeInitEvent) -> Result<(), HandshakeError> {
        self.transport.send(RuntimeMessage::InitEvent(event))?;
        Ok(())
    }

    pub fn complete(mut self) -> Result<Running<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Finished)?;

        Ok(Running {
            transport: self.transport,
        })
    }
}

// --- running ---
pub struct Running<T> {
    transport: T,
}

impl<T> Running<T>
where
    T: AgentTransport,
{
    pub fn recv_request(&mut self) -> Result<Option<RuntimeRequest>, HandshakeError> {
        match self.transport.try_recv()? {
            Some(HandleMessage::Request(req)) => Ok(Some(req)),
            Some(other) => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
            None => Ok(None),
        }
    }

    pub fn send_report(&mut self, report: RuntimeReport) -> Result<(), HandshakeError> {
        self.transport
            .send(RuntimeMessage::Report(Box::new(report)))?;
        Ok(())
    }
}
