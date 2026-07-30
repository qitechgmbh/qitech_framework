use std::collections::HashMap;

use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeStatus;

use crate::MachineEntry;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Status,
    Content,
}

#[derive(Clone, Copy)]
pub struct AppContext {
    pub rt_status: RuntimeStatus,
    pub schemas: *const HashMap<MachineIdentification, MachineSchema>,
    pub machines: *const [MachineEntry],
}

impl AppContext {
    pub fn machines(&self) -> &[MachineEntry] {
        unsafe { &*self.machines }
    }
}

pub enum AppAction {
    NoAction,
    SetConfig {
        machine: MachineIdentificationUnique,
        resource: String,
        value: String,
    },
}
