use std::fmt::{self, Display, Formatter};

use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use control_core::{LogOrigin, MachineIdentificationUnique};

use crate::DataStore;
use crate::data::LogRecorderHandle;
use crate::machine::Hardware ;

mod hardware;
mod config;
mod state;
mod measurement;
mod event;

type BuildResult<T> = Result<T, MachineBuildError>;

pub trait MachineBuild: Sized {
    fn build(builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError>;
}

pub struct MachineBuilder<'a> {
    ident: MachineIdentificationUnique,
    hardware: Vec<Hardware>,
    ethercat_interface: Option<EtherCATThreadChannel>,
    data_store: &'a mut DataStore,
}

impl<'a> MachineBuilder<'a> {
    pub fn new(
        ident: MachineIdentificationUnique,
        hardware: Vec<Hardware>,
        ethercat_interface: Option<EtherCATThreadChannel>,
        data_store: &'a mut DataStore,
    ) -> Self {
        Self {
            ident,
            hardware,
            ethercat_interface,
            data_store,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }

    pub fn log_handle(&mut self) -> LogRecorderHandle {
        let rec = &mut self.data_store.recorder;
        rec.create_log_handle(LogOrigin::Machine(self.ident))
    }
}

// Error
#[derive(Debug)]
pub enum MachineBuildError {
    // --- hardware errors ---
    ExpectedEtherCATInterface,
    ExpectedHardwareAtIndex { index: usize },
    ExpectedEtherCATDeviceWithRole { role: u16 },
    ExpectedEtherCATDeviceAtIndex { index: usize },
    ExpectedSerialDeviceAtIndex { index: usize },
    DeviceTypeMismatch { index: usize, expected: &'static str },
    // --- property errors ---
    AlreadyRegistered { prefix: &'static str, name: &'static str },
    SchemaViolation,
    // --- custom ---
    Custom(anyhow::Error),
}

impl Display for MachineBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered { prefix, name } => {
                write!(f, "'{prefix}.{name}' already registered")
            }
            Self::SchemaViolation => {
                write!(f, "machine schema violation")
            }
            Self::ExpectedEtherCATInterface => {
                write!(f, "machine required a valid ethercat interface")
            }
            Self::ExpectedHardwareAtIndex { index } => {
                write!(f, "expected hardware at index {index}")
            }
            Self::ExpectedEtherCATDeviceWithRole { role } => {
                write!(f, "expected an ethercat device with role {role}")
            }
            Self::ExpectedEtherCATDeviceAtIndex { index } => {
                write!(f, "expected an ethercat device at index {index}")
            }
            Self::ExpectedSerialDeviceAtIndex { index } => {
                write!(f, "expected a serial device at index {index}")
            }
            Self::DeviceTypeMismatch { index, expected } => {
                write!(f, "device type mismatch at index {index}. Expected: {expected}")
            }
            Self::Custom(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for MachineBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Custom(err) => Some(err.root_cause()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for MachineBuildError {
    fn from(err: anyhow::Error) -> Self {
        Self::Custom(err)
    }
}
