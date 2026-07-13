use std::fmt;
use serde::{Deserialize, Serialize};

// --- unique ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentificationUnique {
    pub vendor: u16,
    pub machine: u16,
    pub serial: u32,
}

impl MachineIdentificationUnique {
    pub const fn new(vendor: u16, machine: u16, serial: u32) -> Self {
        Self { vendor, machine, serial }
    }

    pub const fn to_u64(self) -> u64 {
        ((self.vendor as u64) << 48) | ((self.machine as u64) << 32) | (self.serial as u64)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            vendor: (value >> 48) as u16,
            machine: (value >> 32) as u16,
            serial: value as u32,
        }
    }
}

impl fmt::Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.vendor, self.machine, self.serial)
    }
}

// --- non-unique ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentification {
    pub vendor: u16,
    pub machine: u16,
}

impl MachineIdentification {
    pub const fn new(vendor: u16, machine: u16) -> Self {
        Self { vendor, machine }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.vendor as u32) << 16) | (self.machine as u32)
    }

    pub const fn from_u32(value: u32) -> Self {
        Self {
            vendor: (value >> 16) as u16,
            machine: value as u16,
        }
    }
}

impl From<MachineIdentificationUnique> for MachineIdentification {
    fn from(value: MachineIdentificationUnique) -> Self {
        Self { vendor: value.vendor, machine: value.machine }
    }
}

impl fmt::Display for MachineIdentification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.vendor, self.machine)
    }
}
