use crate::RuntimeInitEvent;
use crate::RuntimeReport;
use crate::RuntimeRequest;
use crate::link::error::RuntimeHandleHandshakeError;
use crate::link::error::SchemaSyncError;
use crate::link::protocol::Hello;

pub struct AwaitHello<T> {
    transport: T,
}

impl<T> AwaitHello<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn recv_hello(self) -> Result<SyncSchemas<T>, RuntimeHandleHandshakeError> {
        Ok(SyncSchemas {
            transport: self.transport,
        })
    }
}

pub struct SyncSchemas<T> {
    transport: T,
}

impl<T> SyncSchemas<T> {
    pub fn recv_schema<F>(
        mut self,
        schema: String,
        mut process: F,
    ) -> Result<AwaitEvents<T>, RuntimeHandleHandshakeError>
    where
        F: FnMut(&str) -> Result<(), SchemaSyncError>,
    {
        match process(&schema) {
            Ok(_) => self.send_ack()?,
            Err(_) => todo!(),
        }

        Ok(AwaitEvents {
            transport: self.transport,
        })
    }

    fn send_ack(&mut self) -> Result<(), RuntimeHandleHandshakeError> {
        todo!()
    }
}

pub struct AwaitEvents<T> {
    transport: T,
}

pub struct RuntimeHandle<T> {
    transport: T,
}
