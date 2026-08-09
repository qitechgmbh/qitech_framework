use qitech_framework_core::report::OperationCapability;

use crate::machine::ActResult;
use crate::machine::Machine;

// --- functions ---
pub(crate) type CommandCanExecuteFn = Box<dyn Fn(&dyn Machine) -> OperationCapability>;
pub(crate) type CommandExecuteFn = Box<dyn Fn(&mut dyn Machine) -> ActResult>;

// --- handle ---
pub(crate) struct CommandHandle {
    pub(crate) capability_prev: OperationCapability,
    pub(crate) can_execute_fn: Option<CommandCanExecuteFn>,
    pub(crate) execute_fn: CommandExecuteFn,
}
