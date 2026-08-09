use std::collections::HashMap;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::OperationCapability;

use crate::machine::Machine;
use crate::machine::error::ActResult;
use crate::resource::LifetimeTokenOwner;

pub(crate) struct MachineInstance {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) machine: Box<dyn Machine>,
    pub(crate) configs: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) commands: HashMap<&'static str, CommandHandle>,
    pub(crate) subscriptions: HashMap<MachineIdentificationUnique, LifetimeTokenOwner>,
}

// --- config handle ---
pub type ConfigPropertyWriteFn = Box<dyn Fn(ScalarValue) -> Result<bool, ConfigPropertyWriteError>>;
pub type ConfigPropertyChangedCallbackFn = Box<dyn Fn(&mut dyn Machine) -> ActResult>;

pub(crate) struct ConfigPropertyHandle {
    pub(crate) write: ConfigPropertyWriteFn,
    pub(crate) on_changed: Option<ConfigPropertyChangedCallbackFn>,
}

// --- command handle ---
pub(crate) type CommandCanExecuteFn = Box<dyn Fn(&dyn Machine) -> OperationCapability>;
pub(crate) type CommandExecuteFn = Box<dyn Fn(&mut dyn Machine) -> ActResult>;

pub(crate) struct CommandHandle {
    pub(crate) capability_prev: OperationCapability,
    pub(crate) can_execute_fn: Option<CommandCanExecuteFn>,
    pub(crate) execute_fn: CommandExecuteFn,
}
