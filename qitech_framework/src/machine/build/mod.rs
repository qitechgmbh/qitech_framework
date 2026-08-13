use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr::NonNull;
use std::rc::Rc;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::schema::MachineSchema;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::CommandHandle;
use crate::machine::ConfigPropertyHandle;
use crate::machine::hardware::Hardware;
use crate::resource::Journals;
use crate::resource::PropertyRegistrar;

mod command;
mod config;
mod event;
mod hardware;
mod measurements;
mod state_property;

pub struct BuildContext<'a> {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) schema: &'a MachineSchema,
    pub(crate) export_count: Rc<Cell<u64>>,

    /// Type ID of the machine, used to validate builders that accept `M: Machine`.
    pub(crate) type_id: TypeId,

    /// Fully qualified type name of the machine.
    pub(crate) type_name: &'static str,

    pub(crate) ethercat_interface: Option<EtherCATThreadChannel>,
    pub(crate) hardware: Vec<Hardware>,

    pub(crate) journals: &'a mut Journals,

    /// Journals used to record events during machine initialization.
    ///
    /// These entries are kept separate from the export journals so that
    /// initialization events are not exported if machine construction fails.
    pub(crate) journals_temp: Journals,

    pub(crate) config: PropertyRegistrar<'a>,
    pub(crate) state: PropertyRegistrar<'a>,
    pub(crate) measurements: PropertyRegistrar<'a, unsafe fn(NonNull<()>) -> Option<f64>>,

    pub(crate) config_registered: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) state_registered: HashSet<&'static str>,
    pub(crate) measurements_registered: HashSet<&'static str>,
    pub(crate) commands_registered: HashMap<&'static str, CommandHandle>,
    pub(crate) events_registered: HashSet<&'static str>,
}

impl<'a> BuildContext<'a> {
    pub fn ident(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
