use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::Resources;

pub struct ReactContext<'a> {
    resources: &'a Resources,
}

impl<'a> ReactContext<'a> {
    pub fn read<T>(&self, handle: T) {}
}

impl<'a> ReactContext<'a> {
    pub fn new(resources: &'a Resources) -> Self {
        Self { resources }
    }
}

pub struct SubscribeContext<'a> {
    source: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub fn new(
        source: MachineIdentificationUnique,
        resources: &'a mut Resources,
    ) -> Self {
        Self { source, resources }
    }

    pub fn source(&self) -> MachineIdentificationUnique {
        self.source
    }
}
