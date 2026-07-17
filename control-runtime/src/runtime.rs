use crate::{
    MachineRegistry, 
    data::DataStore, 
    machine::MachineHardwareRegistry
};

pub struct Runtime {
    registry: MachineRegistry,
    machines: Vec<(MachineIdentificationUnique, Box<dyn Machine>)>,
    hardware: MachineHardwareRegistry,
    controller: ethercat::Controller,
    data_store: DataStore,
    // session: Session with hub
}