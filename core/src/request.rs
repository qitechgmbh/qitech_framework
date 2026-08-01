use serde::Deserialize;
use serde::Serialize;

use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequest {
    /// identifier the controller can use to map request back to response
    pub request_id: u64,

    /// the actual request
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
        value: ScalarValue,
    },

    InvokeMachineCommand {
        /// target machine
        target: MachineIdentificationUnique,

        /// command resource path
        resource: String,
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
