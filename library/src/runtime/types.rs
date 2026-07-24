use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use qitech_framework_common::MachineIdentification;
use qitech_framework_common::MachineSchema;
use qitech_lib::ethercat_hal::EtherCATControl;
use qitech_lib::ethercat_hal::Mailbox;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::TripleBufConsumer;
use qitech_lib::ethercat_hal::devices::EthercatDevice;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::error::BuildResult;

pub type MachineRegistry = HashMap<MachineIdentification, MachineRegistryEntry>;

pub struct MachineRegistryEntry {
    #[allow(unused)]
    pub schema: MachineSchema,
    pub build: fn(BuildContext<'_>) -> BuildResult<Box<dyn Machine>>,
}

pub type EtherCATController = EtherCATControl<TripleBufConsumer, Arc<Mailbox>>;
pub type EtherCATSubDevice = (MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>);
