use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::Resources;

pub struct SyncContext<'a> {
    resources: &'a Resources,
}

impl<'a> SyncContext<'a> {
    pub fn read<T>(&self, handle: T) {}
}

impl<'a> SyncContext<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self { resources }
    }
}

pub struct SubscribeContext<'a> {
    source: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub fn new(source: MachineIdentificationUnique, resources: &'a mut Resources) -> Self {
        Self { source, resources }
    }

    pub fn source(&self) -> MachineIdentificationUnique {
        self.source
    }
}
