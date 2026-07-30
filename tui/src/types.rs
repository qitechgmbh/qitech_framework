use std::collections::HashMap;

use qitech_framework::{MachineIdentification, MachineIdentificationUnique};
use qitech_framework_common::{MachineSchema, RuntimeStatus};

use crate::MachineEntry;
use crate::pages::PageId;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Status,
    Menu,
    Content,
}

pub struct AppContext<'a> {
    pub focus: Focus,
    pub page: PageId,
    pub rt_status: RuntimeStatus,
    pub schemas: &'a HashMap<MachineIdentification, MachineSchema>,
    pub machines: &'a [MachineEntry],
}

pub enum AppAction {
    NoAction,
    GotoPage(PageId),
    SetConfig {
        machine: MachineIdentificationUnique,
        resource: String,
        value: String,
    },
}
