use serde::Deserialize;
use serde::Serialize;

use crate::RuntimeInitEvent;
use crate::RuntimeReport;
use crate::RuntimeRequest;
use crate::link::error::HelloMatchError;

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

    pub fn r#match(self, other: Hello) -> Result<(), HelloMatchError> {
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

#[derive(Debug, Serialize, Deserialize)]
pub enum HelloAck {
    Accepted,
    Rejected(HelloMatchError),
}

#[derive(Debug)]
pub enum RuntimeMessage {
    Hello(Hello),
    Schema(String),
    InitEvent(RuntimeInitEvent),
    Finished,
    Report(RuntimeReport),
}

#[derive(Debug)]
pub enum HandleMessage {
    HelloAck,
    SchemaAck,
    SchemaReject(String),
    Request(RuntimeRequest),
}
