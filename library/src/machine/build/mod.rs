use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::journal::Journals;
use crate::machine::ResourceRegistry;
use crate::machine::hardware::Hardware;
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
    ident: MachineIdentificationUnique,

    /// type id of the machine, used for validating builders that accept <M>
    type_id: TypeId,
    type_name: &'static str,

    ethercat_interface: Option<EtherCATThreadChannel>,
    hardware: Vec<Hardware>,

    pub(crate) journals: &'a mut Journals,
    pub(crate) config: PropertyRegistrar<'a>,
    pub(crate) state: PropertyRegistrar<'a>,
    pub(crate) measurements: PropertyRegistrar<'a>,

    journals_temp: Journals,
    // commands: Vec<CommandItem>,
    // events: EventRegistrar<'a>,

    pub(crate) config_registered: HashMap<&'static str, ConfigPropertyWriteFn>,
    pub(crate) state_registered: HashSet<&'static str>,
    pub(crate) measurements_registered: HashSet<&'static str>,
}

impl<'a> BuildContext<'a> {
    pub(crate) fn new(
        ident: MachineIdentificationUnique,
        type_id: TypeId,
        type_name: &'static str,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        journals: &'a mut Journals,
        resources: &'a mut ResourceRegistry,
    ) -> Self {
        // let events = EventRegistrar::new(&mut resources.events, &mut journals.events, ident);

        Self {
            ident,
            type_id,
            type_name,
            ethercat_interface,
            hardware,
            config: resources.config_properties.register(),
            state: resources.state_properties.register(),
            measurements: resources.measurements.register(),
            journals,
            journals_temp: Journals::default(),
            config_registered: Default::default(),
            state_registered: Default::default(),
            measurements_registered: Default::default(),
        }
    }

    pub(crate) fn commit_all(self) {
        self.config.commit();
        self.state.commit();
        self.measurements.commit();
    }

    pub fn ident(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
