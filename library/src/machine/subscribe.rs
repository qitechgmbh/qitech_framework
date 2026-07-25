use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::Resources;

pub struct ReactContext<'a> {
    resources: &'a mut Resources,
}

impl<'a> ReactContext<'a> {
    pub fn read<T>(&self, handle: T) {}
}

pub struct SubscribeContext<'a> {
    source: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub fn source(&self) -> MachineIdentificationUnique {
        self.source
    }
}
