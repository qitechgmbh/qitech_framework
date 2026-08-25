use std::collections::HashMap;

use qitech_framework::machine::OperationCapability;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EventRecord;
use qitech_framework_core::report::StatePropertyEvent;

#[derive(Default, Clone)]
pub struct MachineInstance {
    pub config_properties: HashMap<String, Option<ConfigPropertyInfo>>,
    pub state_properties: HashMap<String, Option<StatePropertyInfo>>,
    pub measurements: HashMap<String, Option<MeasurementInfo>>,
}

#[derive(Clone)]
pub struct ConfigPropertyInfo {
    pub value: ScalarValue,
    pub default: ScalarValue,
    pub capability: OperationCapability,
    pub constraints: Constraints,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,
}

#[derive(Clone)]
pub struct StatePropertyInfo {
    pub value: ScalarValue,
    pub records: Vec<EventRecord<StatePropertyEvent>>,
}

#[derive(Clone)]
pub struct MeasurementInfo {
    pub value: Option<f64>,
}
