use control_core::MachineIdentificationUnique;

use crate::DataRegistry;





pub struct AttachmentContext<'a> {
    registry: &'a DataRegistry,
    ident: MachineIdentificationUnique,
    config: ConfigPropertyResolver,
    state: ConfigPropertyResolver,
    measurements: ConfigPropertyResolver,
}

pub struct ConfigPropertyResolver {

}

impl ConfigPropertyResolver {
    pub fn resolve(&mut self) ->  {

    }
}

pub struct ConfigPropertyAccessor {
    generation: u64,
    index: usize,
}

pub struct StatePropertyResolver {

}

pub struct MeasurementResolver {

}