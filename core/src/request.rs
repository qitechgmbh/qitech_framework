use serde::Deserialize;
use serde::Serialize;

use crate::ident::MachineIdentificationUnique;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequest {
    pub transaction_id: u64,
    pub kind: RuntimeRequestKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeRequestKind {
    WriteMachineDeviceInfo {
        /// machine hardware identification
        machine_ident: MachineIdentificationUnique,

        /// role of the device
        role: u16,

        /// ethercat hardware identification
        subdevice_index: usize,
    },

    SetMachineConfiguration {
        /// target machine
        target: MachineIdentificationUnique,

        /// resource path
        resource: String,

        /// value to write
        value: String,
    },

    InvokeMachineCommand {
        /// target machine
        target: MachineIdentificationUnique,

        /// command resource path
        resource: String,

        /// command arguments
        arguments: String,
    },

    MachineSubscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },

    MachineUnsubscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
}
