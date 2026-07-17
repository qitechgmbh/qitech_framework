use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::MachineIdentificationUnique;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogRecord {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub origin: LogOrigin,
    pub message: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogOrigin {
    Hub,
    Runtime,
    Machine(MachineIdentificationUnique),
}

impl LogOrigin {
    pub fn to_u64(self) -> u64 {
        match self {
            LogOrigin::Hub => 0,
            LogOrigin::Runtime => 1,
            LogOrigin::Machine(ident) => ident.to_u64(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum LogLevel {
    Trace   = 0,
    Debug   = 1,
    Info    = 2,
    Warn    = 3,
    Error   = 4,
}
