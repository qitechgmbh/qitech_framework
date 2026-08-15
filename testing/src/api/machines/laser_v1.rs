use qitech_framework::MachineIdentificationUnique;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::request::RuntimeRequestKind;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub diameter: f64,
    pub x_diameter: Option<f64>,
    pub y_diameter: Option<f64>,
    pub roundness: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StateEvent {
    pub is_default_state: bool,
    pub laser_state: LaserState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LaserState {
    pub higher_tolerance: f64,
    pub lower_tolerance: f64,
    pub target_diameter: f64,
    pub in_tolerance: bool,
    pub global_warning: bool,
}

// --- mutations ---
#[derive(Deserialize, Serialize)]
pub enum Mutation {
    SetTargetDiameter(f64),
    SetLowerTolerance(f64),
    SetHigherTolerance(f64),
    SetGlobalWarning(bool),
}

impl Mutation {
    pub fn into_request(self, target: MachineIdentificationUnique) -> RuntimeRequestKind {
        match self {
            Mutation::SetTargetDiameter(v) => RuntimeRequestKind::SetConfigProperty {
                target,
                path: "diameter.target".to_string(),
                value: ScalarValue::Float(v),
            },

            Mutation::SetLowerTolerance(v) => RuntimeRequestKind::SetConfigProperty {
                target,
                path: "diameter.lower_tolerance".to_string(),
                value: ScalarValue::Float(v),
            },

            Mutation::SetHigherTolerance(v) => RuntimeRequestKind::SetConfigProperty {
                target,
                path: "diameter.higher_tolerance".to_string(),
                value: ScalarValue::Float(v),
            },

            Mutation::SetGlobalWarning(v) => RuntimeRequestKind::SetConfigProperty {
                target,
                path: "warning.global".to_string(),
                value: ScalarValue::Boolean(v),
            },
        }
    }
}
