use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::resource::CommandManager;
use crate::machine::resource::ConfigPropertyManager;
use crate::machine::resource::EventManager;
use crate::machine::resource::MeasurementManager;
use crate::machine::resource::ResourceKind;
use crate::machine::resource::StatePropertyManager;

pub struct ResourceManager {
    resources: Vec<ResourceEntry>,



    // --- managers ---
    config_properties: ConfigPropertyManager,
    state_properties: StatePropertyManager,
    measurements: MeasurementManager,
    commands: CommandManager,
    events: EventManager,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceKey {
    machine: MachineIdentificationUnique,
    resource: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ResourceEntry {
    machine: MachineIdentificationUnique,
    path: &'static str,
    kind: ResourceKind,
    id: ResourceId,
}

type ResourceId = u64;
