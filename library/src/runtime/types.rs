use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATControl;
use qitech_lib::ethercat_hal::Mailbox;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::TripleBufConsumer;
use qitech_lib::ethercat_hal::devices::EthercatDevice;

use crate::machine::BuildContext;
use crate::machine::Hardware;
use crate::machine::Machine;
use crate::machine::error::BuildResult;

pub type HardwareRegistry = HashMap<MachineIdentificationUnique, Vec<Hardware>>;
pub type MachineRegistry = HashMap<MachineIdentification, BuildMachineFn>;
pub type BuildMachineFn = fn(BuildContext<'_>) -> BuildResult<Box<dyn Machine>>;

pub type EtherCATController = EtherCATControl<TripleBufConsumer, Arc<Mailbox>>;
pub type EtherCATSubDevice = (MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>);

pub struct Config {
    pub requests_per_cycle_max: usize,
    pub export_interval: Duration,
    pub cycle_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            requests_per_cycle_max: 10,
            export_interval: Duration::from_secs_f64(1.0 / 30.0),
            cycle_timeout: Duration::from_micros(100),
        }
    }
}

pub struct MachineInstance {
    pub ident: MachineIdentificationUnique,
    pub inner: Box<dyn Machine>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Initialized,
    Running,
    Stopped,
    Failed,
}
