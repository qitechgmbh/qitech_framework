use qitech_framework_common::MachineIdentificationUnique;
use crate::machine::resource::state_property;
use crate::machine::resource::config_property;
use crate::machine::resource::measurement;

pub struct ReactContext<'a> {
    pub config: config_property::Reader<'a>,
    pub state: state_property::Reader<'a>,
    pub measurements: measurement::Reader<'a>,
}

pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: config_property::Resolver<'a>,
    pub state: state_property::Resolver<'a>,
    pub measurements: measurement::Resolver<'a>,
}
