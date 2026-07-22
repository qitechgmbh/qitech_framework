use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::fmt;

use control_core::{LogRecord, MachineConfigMutation, MachineEvent, MachineStateMutation};

mod types;

mod registry;
pub use registry::MachineResourceRegistry;

mod log;
mod property;
mod measurement;

const MACHINE_NAMES_COUNT_MAX: usize = 2048;
const MACHINE_NAME_LEN_MAX: usize = 96;

const MACHINE_MEASUREMENTS_REGISTRY_ID: usize = 0;
const MACHINE_MEASUREMENTS_COUNT_MAX: usize = 512;

const MACHINE_CONFIG_PROPERTIES_REGISTRY_ID: usize = 1;
const MACHINE_CONFIG_PROPERTIES_COUNT_MAX: usize = 512;

const MACHINE_STATE_PROPERTIES_REGISTRY_ID: usize = 2;
const MACHINE_STATE_PROPERTIES_COUNT_MAX: usize = 512;

// --- name registry ---

#[derive(Debug, Default)]
pub struct NameRegistry(heapless::FnvIndexMap<&'static str, &'static str, MACHINE_NAMES_COUNT_MAX>);

impl NameRegistry {
    /// Interns a name: returns a 'static lifetime version of input &str.
    /// Achieved by keeping a registry of all registered names, which are
    /// behind the scenes leaked strings. Bounded by the vec limit, 
    /// so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    fn register_name(&mut self, name: &str) -> Result<&'static str, ResourceRegisterError> {
        let reg = &mut self.0;

        if name.len() > MACHINE_NAME_LEN_MAX {
            return Err(ResourceRegisterError::NameTooLarge { name: name.to_string() });
        }

        if let Some(&existing) = reg.get(name) {
            return Ok(existing);
        }

        if reg.len() >= reg.capacity() {
            return Err(ResourceRegisterError::NameRegistryFull { name: name.to_string() })
        }

        // entry not found, create a new one by leaking the address
        let leaked: &'static str = name.to_string().leak();
        reg.insert(leaked, leaked);
        Ok(leaked)
    }
}

// --- config properties ---
pub type MachineConfigPropertyRegistry = 
    property::Registry<MACHINE_CONFIG_PROPERTIES_REGISTRY_ID, MACHINE_CONFIG_PROPERTIES_COUNT_MAX>;

pub type MachineConfigPropertyResolver<'a> = 
    property::Resolver<'a, MACHINE_CONFIG_PROPERTIES_REGISTRY_ID, MACHINE_CONFIG_PROPERTIES_COUNT_MAX>;

pub type MachineConfigPropertyReader<'a> = 
    property::Reader<'a, MACHINE_CONFIG_PROPERTIES_REGISTRY_ID, MACHINE_CONFIG_PROPERTIES_COUNT_MAX>;

pub type MachineConfigPropertyAccessHandle<T> = 
    property::ReaderHandle<MACHINE_CONFIG_PROPERTIES_REGISTRY_ID, T>;

pub type MachineConfigPropertyHandle<T> = property::Handle<T>;

// --- state properties ---
pub type MachineStatePropertyRegistry = 
    property::Registry<MACHINE_STATE_PROPERTIES_REGISTRY_ID, MACHINE_STATE_PROPERTIES_COUNT_MAX>;

pub type MachineStatePropertyResolver<'a> = 
    property::Resolver<'a, MACHINE_STATE_PROPERTIES_REGISTRY_ID, MACHINE_STATE_PROPERTIES_COUNT_MAX>;

pub type MachineStatePropertyReader<'a> = 
    property::Reader<'a, MACHINE_STATE_PROPERTIES_REGISTRY_ID, MACHINE_STATE_PROPERTIES_COUNT_MAX>;

pub type MachineStatePropertyAccessHandle<T> = 
    property::ReaderHandle<MACHINE_STATE_PROPERTIES_REGISTRY_ID, T>;

pub type MachineStatePropertyHandle<T> = property::Handle<T>;

// --- measurements ---
pub type MachineMeasurementRegistry = 
    measurement::Registry<MACHINE_MEASUREMENTS_REGISTRY_ID, MACHINE_MEASUREMENTS_COUNT_MAX>;

pub type MachineMeasurementResolver<'a> = 
    measurement::Resolver<'a, MACHINE_MEASUREMENTS_REGISTRY_ID, MACHINE_MEASUREMENTS_COUNT_MAX>;

pub type MachineMeasurementReader<'a> = 
    measurement::Reader<'a, MACHINE_MEASUREMENTS_REGISTRY_ID, MACHINE_MEASUREMENTS_COUNT_MAX>;

pub type MachineMeasurementAccessHandle<T> = 
    property::ReaderHandle<MACHINE_MEASUREMENTS_REGISTRY_ID, T>;

pub type MachineMeasurementHandle = measurement::Handle;

pub type ResourceJournal<T> = Rc<RefCell<Vec<T>>>;
pub type WeakResourceJournal<T> = Weak<RefCell<Vec<T>>>;

#[derive(Debug, Default)]
pub struct ResourceJournals {
    config: ResourceJournal<MachineConfigMutation>,
    state:  ResourceJournal<MachineStateMutation>,
    event:  ResourceJournal<MachineEvent>,
    logs:   ResourceJournal<LogRecord>,
}

impl ResourceJournals {
    pub fn new() -> Self { Self::default() }
}

pub enum ResourceRegisterError {
    NameTooLarge { 
        name: String,
    },
    NameRegistryFull {
        name: String 
    },
    RegistryFull { 
        name: &'static str 
    },
    AlreadyRegistered { 
        name: &'static str 
    },
    TypeTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
    AlignmentTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceResolveError {
    NoSuchProperty,
    InvalidType,
}

#[derive(Debug)]
pub struct ResourceReadError;

impl fmt::Display for ResourceReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expired handle")
    }
}

impl std::error::Error for ResourceReadError {}