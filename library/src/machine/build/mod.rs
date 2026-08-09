use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ptr::NonNull;
use std::rc::Rc;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::CommandHandle;
use crate::machine::ConfigPropertyHandle;
use crate::machine::hardware::Hardware;
use crate::resource::Journals;
use crate::resource::PropertyRegistrar;

mod command;
mod config;
// mod event;
mod hardware;
mod measurements;
mod state_property;
// mod resource;

pub struct BuildContext<'a> {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) export_count: Rc<Cell<u64>>,

    /// type id of the machine, used for validating builders that accept <M: Machine>
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,

    pub(crate) ethercat_interface: Option<EtherCATThreadChannel>,
    pub(crate) hardware: Vec<Hardware>,

    pub(crate) journals: &'a mut Journals,

    // owned set of journals for recording during initiliazation.
    // useful so we can record entries without putting them into the export
    // journal in case the machine fails and we don't want to send out the events
    pub(crate) journals_temp: Journals,

    pub(crate) config: PropertyRegistrar<'a>,
    pub(crate) state: PropertyRegistrar<'a>,
    pub(crate) measurements: PropertyRegistrar<'a, unsafe fn(NonNull<()>) -> Option<f64>>,

    pub(crate) config_registered: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) state_registered: HashSet<&'static str>,
    pub(crate) measurements_registered: HashSet<&'static str>,
    pub(crate) commands_registered: HashMap<&'static str, CommandHandle>,
}

impl<'a> BuildContext<'a> {
    pub fn ident(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
