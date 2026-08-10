use crate::report::RuntimeInitEvent;
use crate::report::RuntimeReport;
use crate::request::RuntimeRequest;
use crate::schema::MachineSchema;
use crate::session::error::HandshakeError;
use crate::session::error::SessionRecvError;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::Hello;
use crate::session::protocol::RuntimeMessage;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

// --- send hello ---
pub struct SessionHandshake<T>
where
    T: RuntimeTransport,
{
    transport: T,
}

impl<T> SessionHandshake<T>
where
    T: RuntimeTransport,
{
    pub(crate) fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T> SessionHandshake<T>
where
    T: RuntimeTransport,
{
    pub fn begin_sync(mut self) -> Result<SessionSyncingSchemas<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Hello(Hello::new()))?;

        match self.transport.recv()? {
            ControllerMessage::HelloAck => Ok(SessionSyncingSchemas {
                transport: self.transport,
            }),

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
    T: RuntimeTransport,
{
    pub fn sync(&mut self, schema: MachineSchema) -> Result<(), HandshakeError> {
        self.transport
            .send(RuntimeMessage::Schema(Box::new(schema)))?;

        match self.transport.recv()? {
            ControllerMessage::SchemaAck => Ok(()),

            ControllerMessage::SchemaReject(reason) => Err(HandshakeError::SchemaRejected(reason)),

            other => Err(HandshakeError::UnexpectedMessage(format!("{other:?}"))),
        }
    }

    pub fn begin_initialization(mut self) -> Result<SessionInitializing<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Finished)?;

        Ok(SessionInitializing {
            transport: self.transport,
        })
    }
}

// --- send init events ---
pub struct SessionInitializing<T> {
    transport: T,
}

impl<T> SessionInitializing<T>
where
    T: RuntimeTransport,
{
    pub fn send_event(&mut self, event: RuntimeInitEvent) -> Result<(), HandshakeError> {
        self.transport.send(RuntimeMessage::InitEvent(event))?;
        Ok(())
    }

    pub fn upgrade(mut self) -> Result<SessionRunning<T>, HandshakeError> {
        self.transport.send(RuntimeMessage::Finished)?;

        // don't block from now on
        self.transport.set_blocking(false)?;

        Ok(SessionRunning {
            transport: self.transport,
        })
    }
}

// --- run ---
pub struct SessionRunning<T> {
    transport: T,
}

impl<T> SessionRunning<T>
where
    T: RuntimeTransport,
{
    pub fn recv_request(&mut self) -> Result<Option<RuntimeRequest>, SessionRecvError> {
        match self.transport.recv() {
            Ok(ControllerMessage::Request(req)) => Ok(Some(req)),
            Ok(other) => Err(SessionRecvError::UnexpectedMessage(format!("{other:?}"))),
            Err(TransportError::WouldBlock) => Ok(None),
            Err(e) => Err(SessionRecvError::Transport(e)),
        }
    }

    pub fn send_report(&mut self, report: RuntimeReport) -> Result<(), TransportError> {
        self.transport
            .send(RuntimeMessage::Report(Box::new(report)))?;
        Ok(())
    }
}
