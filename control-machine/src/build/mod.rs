/*
pub trait MachineBuild: Sized {
    fn build(ctx: MachineBuildContext<'_>) -> Result<Self, MachineBuildError>;
}

pub struct MachineBuildContext<'a> {
    ident: MachineIdentificationUnique,
    hardware: Vec<Hardware>,
    ethercat_interface: Option<EtherCATThreadChannel>,
    resource_registry: &'a mut MachineResourceRegistry,
    resource_journals: &'a mut ResourceJournals,
}
*/