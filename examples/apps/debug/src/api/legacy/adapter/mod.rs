use qitech_framework::MachineInstanceIdentification;
use qitech_framework_core::request::RuntimeRequestKind;

use crate::api::types::MachineInstance;

mod laser_v1;
pub use laser_v1::LASER_V1;

#[derive(Clone)]
pub struct MachineLegacyDataAdapter {
    pub convert_request: fn(
        MachineInstanceIdentification,
        serde_json::Value,
    ) -> Result<RuntimeRequestKind, serde_json::Error>,

    pub init_state_event: fn(&MachineInstance) -> serde_json::Value,
    pub init_measurements_event: fn(&MachineInstance) -> serde_json::Value,
}
