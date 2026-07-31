use crate::RuntimeInitEvent;
use crate::RuntimeReport;
use crate::RuntimeRequest;
use crate::link::Transport;
use crate::link::error::RuntimeHandleHandshakeError;
use crate::link::protocol::HandleMessage;
use crate::link::protocol::Hello;
use crate::link::protocol::RuntimeMessage;

pub struct SendHello<T> {
    transport: T,
}

impl<T> SendHello<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T> SendHello<T>
where
    T: Transport<HandleMessage, RuntimeMessage>,
{
    pub fn execute(mut self, hello: Hello) -> Result<SyncSchemas<T>, RuntimeHandleHandshakeError> {
        self.transport.send(RuntimeMessage::Hello(hello))?;

        match self.transport.recv()? {
            HandleMessage::HelloAck => Ok(SyncSchemas {
                transport: self.transport,
            }),

            other => Err(RuntimeHandleHandshakeError::UnexpectedMessage(format!(
                "{other:?}"
            ))),
        }
    }
}

pub struct SyncSchemas<T> {
    transport: T,
}

impl<T> SyncSchemas<T>
where
    T: Transport<HandleMessage, RuntimeMessage>,
{
    pub fn sync_schema(mut self, schema: String) -> Result<Self, RuntimeHandleHandshakeError> {
        self.transport.send(RuntimeMessage::Schema(schema))?;

        match self.transport.recv()? {
            HandleMessage::SchemaAck => Ok(self),

            HandleMessage::SchemaReject(reason) => {
                Err(RuntimeHandleHandshakeError::SchemaRejected(reason))
            }

            other => Err(RuntimeHandleHandshakeError::UnexpectedMessage(format!(
                "{other:?}"
            ))),
        }
    }

    pub fn finish(self) -> Initializing<T> {
        Initializing {
            transport: self.transport,
        }
    }
}

pub struct Initializing<T> {
    transport: T,
}

impl<T> Initializing<T>
where
    T: Transport<HandleMessage, RuntimeMessage>,
{
    pub fn send_event(
        mut self,
        event: RuntimeInitEvent,
    ) -> Result<Self, RuntimeHandleHandshakeError> {
        self.transport.send(RuntimeMessage::InitEvent(event))?;

        Ok(self)
    }

    pub fn finish(mut self) -> Result<Running<T>, RuntimeHandleHandshakeError> {
        self.transport.send(RuntimeMessage::Finished)?;

        Ok(Running {
            transport: self.transport,
        })
    }
}

pub struct Running<T> {
    transport: T,
}

impl<T> Running<T>
where
    T: Transport<HandleMessage, RuntimeMessage>,
{
    pub fn recv_request(&mut self) -> Result<Option<RuntimeRequest>, RuntimeHandleHandshakeError> {
        match self.transport.try_recv()? {
            Some(HandleMessage::Request(req)) => Ok(Some(req)),
            Some(other) => Err(RuntimeHandleHandshakeError::UnexpectedMessage(format!(
                "{other:?}"
            ))),
            None => Ok(None),
        }
    }

    pub fn send_report(
        &mut self,
        report: RuntimeReport,
    ) -> Result<(), RuntimeHandleHandshakeError> {
        self.transport.send(RuntimeMessage::Report(report))?;

        Ok(())
    }
}
