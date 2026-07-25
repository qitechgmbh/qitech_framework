use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::Machine;
use crate::runtime::types::MachineInstance;

pub fn find_machine(
    machines: &mut Vec<MachineInstance>,
    ident: MachineIdentificationUnique,
) -> Option<&mut dyn Machine> {
    machines
        .iter_mut()
        .find(|(id, _)| *id == ident)
        .map(|(_, m)| m.as_mut())
}