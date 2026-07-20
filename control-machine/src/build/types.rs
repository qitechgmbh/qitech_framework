use std::fmt::{self, Display, Formatter};

use crate::resource::RegisterError;

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
    Custom(String),
}

impl From<RegisterError> for MachineBuildError {
    fn from(value: RegisterError) -> Self {
        use RegisterError::*;
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
            },
            NameTooLarge { name } => { todo!(); },
            NameRegistryFull { name } => { todo!();  },
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
