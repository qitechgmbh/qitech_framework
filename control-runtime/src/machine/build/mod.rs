use std::fmt::{self, Display, Formatter};

use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use control_core::MachineIdentificationUnique;

use crate::resource::{ResourceJournals, ResourceRegisterError, MachineResourceRegistry};
use crate::machine::Hardware ;

mod hardware;
mod config;
mod state;
mod measurement;
mod command;
// mod event;

type BuildResult<T> = Result<T, MachineBuildError>;

pub trait MachineBuild: Sized {
    fn build(ctx: MachineBuildContext<'_>) -> Result<Self, MachineBuildError>;
}

pub struct MachineBuildContext<'a> {
    ident: MachineIdentificationUnique,
    hardware: Vec<Hardware>,
    ethercat_interface: Option<EtherCATThreadChannel>,
    resource_registry: &'a mut MachineResourceRegistry,
    resource_journals: &'a mut ResourceJournals,
}

impl<'a> MachineBuildContext<'a> {
    pub fn new(
        ident: MachineIdentificationUnique,
        resource_registry: &'a mut MachineResourceRegistry,
        resource_journals: &'a mut ResourceJournals,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
    ) -> Self {
        Self {
            ident,
            resource_registry,
            resource_journals,
            ethercat_interface,
            hardware,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }

    pub fn log_handle(&mut self) -> LogRecorderHandle {
        // let rec = &mut self.journals;
        // rec.create_log_handle(LogOrigin::Machine(self.ident))
        todo!()
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
    // --- resource errors ---
    AlreadyRegistered {
        registry: &'static str,
        name: &'static str,
    },
    RegistryFull {
        registry: &'static str,
        name: &'static str,
    },
    TypeTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
    AlignmentTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
    SchemaViolation,
    // --- custom ---
    Custom(anyhow::Error),
}

impl From<ResourceRegisterError> for MachineBuildError {
    fn from(value: resource::RegisterError) -> Self {
        use ResourceRegisterError::*;
        match value {
            AlreadyRegistered { name } => MachineBuildError::AlreadyRegistered { 
                registry: "measurements",  
                name 
            },
            RegistryFull { name } => MachineBuildError::RegistryFull { 
                registry: "measurements",  
                name,  
            },
            TypeTooLarge { r#type, name } => MachineBuildError::TypeTooLarge {
                r#type, 
                name
            },
            AlignmentTooLarge { r#type, name } => MachineBuildError::TypeTooLarge {
                r#type, 
                name,
            }
        }
    }
}

impl Display for MachineBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRegistered { registry, name } => {
                write!(f, "'{registry}.{name}' already registered")
            }
            Self::RegistryFull { registry, name } => {
                write!(f, "failed to register {name}: registry '{registry}' full")
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
            _ => todo!()
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
