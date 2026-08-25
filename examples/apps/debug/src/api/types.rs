use std::collections::HashMap;

use qitech_framework::ConfigPropertyEventRecord;
use qitech_framework::Constraints;
use qitech_framework::ScalarValue;
use qitech_framework::StatePropertyEventRecord;
use qitech_framework::machine::OperationCapability;

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
    pub records: Vec<ConfigPropertyEventRecord>,
}

#[derive(Clone)]
pub struct StatePropertyInfo {
    pub value: ScalarValue,
    pub records: Vec<StatePropertyEventRecord>,
}

#[derive(Clone)]
pub struct MeasurementInfo {
    pub value: Option<f64>,
}
