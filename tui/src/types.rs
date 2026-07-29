use qitech_framework_common::RuntimeStatus;

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
    pub machines: &'a [MachineEntry],
}

pub enum AppAction {
    NoAction,
    GotoPage(PageId),
    Page(PageEvent),
}

pub enum PageEvent {}
