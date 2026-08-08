use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::journal::Journals;
use crate::machine::ConfigPropertyHandle;
use crate::machine::hardware::Hardware;
use crate::machine::instance::ConfigPropertyChangedCallbackFn;
use crate::machine::instance::ConfigPropertyWriteFn;
use crate::machine::property_registry::PropertyRegistrar;

// mod command;
mod config;
// mod event;
mod hardware;
mod measurements;
mod state_property;
// mod resource;

pub struct BuildContext<'a> {
    pub(crate) ident: MachineIdentificationUnique,

    /// type id of the machine, used for validating builders that accept <M>
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,

    pub(crate) ethercat_interface: Option<EtherCATThreadChannel>,
    pub(crate) hardware: Vec<Hardware>,

    pub(crate) journals: &'a mut Journals,
    pub(crate) config: PropertyRegistrar<'a>,
    pub(crate) state: PropertyRegistrar<'a>,
    pub(crate) measurements: PropertyRegistrar<'a>,

    pub(crate) journals_temp: Journals,

    // commands: Vec<CommandItem>,
    // events: EventRegistrar<'a>,
    pub(crate) config_registered: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) state_registered: HashSet<&'static str>,
    pub(crate) measurements_registered: HashSet<&'static str>,
}

impl<'a> BuildContext<'a> {
    pub fn ident(&self) -> MachineIdentificationUnique {
        self.ident
    }
}

// --- registration ---
pub struct ConfigPropertyRegistration {
    pub write_fn: ConfigPropertyWriteFn,
    pub on_external_write_fn: Option<ConfigPropertyChangedCallbackFn>,
}
