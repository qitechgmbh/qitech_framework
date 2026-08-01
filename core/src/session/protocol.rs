use serde::Deserialize;
use serde::Serialize;

use crate::report::RuntimeInitEvent;
use crate::report::RuntimeReport;
use crate::request::RuntimeRequest;
use crate::schema::MachineSchema;
use crate::session::error::HelloMatchError;
use crate::session::error::SchemaSyncError;

const MAGIC: u64 = 0x4855425F4C494E4B;
const PROTOCOL_VERSION: u64 = 0x1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Hello {
    magic: u64,
    protocol_version: u64,
}

impl Hello {
    pub fn new() -> Self {
        Self {
            magic: MAGIC,
            protocol_version: PROTOCOL_VERSION,
        }
    }

    pub fn try_match(self, other: Hello) -> Result<(), HelloMatchError> {
        if self.magic != other.magic {
            return Err(HelloMatchError::MagicMismatch {
                expected: self.magic,
                received: other.magic,
            });
        }

        if self.protocol_version != other.protocol_version {
            return Err(HelloMatchError::ProtocolVersionMismatch {
                expected: self.protocol_version,
                received: other.protocol_version,
            });
        }

        Ok(())
    }
}

impl Default for Hello {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HelloAck {
    Accepted,
    Rejected(HelloMatchError),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeMessage {
    Hello(Hello),
    Schema(Box<MachineSchema>),
    InitEvent(RuntimeInitEvent),
    Finished,
    Report(Box<RuntimeReport>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerMessage {
    HelloAck,
    HelloReject(HelloMatchError),
    SchemaAck,
    SchemaReject(SchemaSyncError),
    Request(RuntimeRequest),
}
