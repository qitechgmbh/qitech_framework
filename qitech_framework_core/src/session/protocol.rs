use serde::Deserialize;
use serde::Serialize;

use crate::report::RuntimeReport;
use crate::request::RuntimeRequest;
use crate::schema::MachineSchema;
use crate::session::error::HelloMatchError;

const MAGIC: u64 = 0x4855425F4C494E4B;
const PROTOCOL_VERSION: u64 = 0x1;

/// Initial handshake payload to ensure the protocol_versions match.
/// This is to meant to have one stable payload to discover payload
/// mismatches instead of simply firing an error that it couldn't be parsed/processed.
///
/// NOTE: EXPECTED TO STAY STABLE. DO NOT CHANGE LAYOUT, EVER
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

    pub fn validate(self) -> Result<(), HelloMatchError> {
        if self.magic != MAGIC {
            return Err(HelloMatchError::MagicMismatch {
                expected: MAGIC,
                received: self.magic,
            });
        }

        if self.protocol_version != PROTOCOL_VERSION {
            return Err(HelloMatchError::ProtocolVersionMismatch {
                expected: PROTOCOL_VERSION,
                received: self.protocol_version,
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
pub struct RuntimeInfo {
    pub schemas: Vec<MachineSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeMessage {
    HelloAck(RuntimeInfo),
    HelloReject(HelloMatchError),
    Report(Box<RuntimeReport>),
    UnexpectedMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControllerMessage {
    Hello(Hello),
    Start,
    Request(RuntimeRequest),
}
