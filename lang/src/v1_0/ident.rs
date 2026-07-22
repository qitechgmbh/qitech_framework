use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentificationUnique {
    pub vendor: u16,
    pub machine: u16,
    pub serial: u16,
}

impl MachineIdentificationUnique {
    pub const fn new(vendor: u16, machine: u16, serial: u16) -> Self {
        Self { vendor, machine, serial }
    }

    pub const fn to_u64(self) -> u64 {
        ((self.vendor as u64) << 48) | ((self.machine as u64) << 32) | (self.serial as u64)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            vendor: (value >> 48) as u16,
            machine: (value >> 32) as u16,
            serial: value as u16,
        }
    }

    pub const fn is_valid(self) -> bool {
        vendors::contains_id(self.vendor) && self.machine != 0 
    }
}

impl fmt::Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.vendor, self.machine, self.serial)
    }
}