use serde::{Deserialize, Serialize};

pub const MAGIC: u64 = 0x4855425F4C494E4B;
pub const PROTOCOL_VERSION: u32 = 0x1;

/// Sent first, by the client, once the socket connects.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub magic: u64,
    pub protocol_version: u32,
}

impl Hello {
    pub fn new() -> Self {
        Self { 
            magic: MAGIC, 
            protocol_version: PROTOCOL_VERSION,
        }
    }
}

impl Default for Hello {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandshakeMessage {
    Start,
    Finish,
    RegisterMachine(String),
}
