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
    pub const fn to_u64(self) -> u64 {
        match self {
            LogOrigin::Hub => 1 << 63,
            LogOrigin::Runtime => 1 << 62,
            LogOrigin::Machine(id) => id.to_u64(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}