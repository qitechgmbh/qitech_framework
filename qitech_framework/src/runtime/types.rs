use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;
use qitech_lib::ethercat_hal::EtherCATControl;
use qitech_lib::ethercat_hal::Mailbox;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::TripleBufConsumer;

use crate::machine::BuildContext;
use crate::machine::CommandHandle;
use crate::machine::ConfigPropertyHandle;
use crate::machine::Hardware;
use crate::machine::Machine;
use crate::resource::LifetimeTokenOwner;

pub(crate) type HardwareRegistry = HashMap<MachineIdentificationUnique, Vec<Hardware>>;
pub(crate) type MachineRegistry = HashMap<MachineIdentification, MachineRegistryEntry>;
pub(crate) type BuildMachineFn =
    fn(&mut BuildContext) -> Result<Box<dyn Machine + 'static>, BuildError>;

pub(crate) type EtherCATController = EtherCATControl<TripleBufConsumer, Arc<Mailbox>>;

pub(crate) struct MachineRegistryEntry {
    pub(crate) schema: MachineSchema,
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,
    pub(crate) build: BuildMachineFn,
}

pub(crate) struct MachineIdentificationPreset {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) vendor_id: u32,
    pub(crate) product_id: u32,
    pub(crate) revision: Option<u32>,
}

impl MachineIdentificationPreset {
    pub(crate) fn matches(&self, meta: &MetaSubdevice) -> bool {
        self.vendor_id == meta.vendor
            && self.product_id == meta.product_id
            && self.revision.is_none_or(|rev| rev == meta.revision)
    }
}

pub(crate) struct MachineInstance {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) machine: Box<dyn Machine>,
    pub(crate) configs: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) commands: HashMap<&'static str, CommandHandle>,
    pub(crate) subscriptions: HashMap<MachineIdentificationUnique, LifetimeTokenOwner>,
}

pub(crate) struct Config {
    pub(crate) requests_per_cycle_max: usize,
    pub(crate) export_interval: Duration,
    pub(crate) cycle_period: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            requests_per_cycle_max: 10,
            export_interval: Duration::from_secs_f64(1.0 / 32.0),
            cycle_period: Duration::from_micros(100),
        }
    }
}
