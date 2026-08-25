use qitech_framework::MachineInstanceIdentification;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::request::RuntimeRequestKind;
use serde::Deserialize;
use serde::Serialize;

use crate::api::legacy::MachineLegacyDataAdapter;
use crate::api::types::MachineInstance;

pub const LASER_V1: MachineLegacyDataAdapter = MachineLegacyDataAdapter {
    convert_request,
    init_state_event,
    init_measurements_event,
};

fn convert_request(
    ident: MachineInstanceIdentification,
    data: serde_json::Value,
) -> Result<RuntimeRequestKind, serde_json::Error> {
    Ok(match serde_json::from_value(data)? {
        Mutation::SetTargetDiameter(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.target".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetLowerTolerance(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.tolerance.lower".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetHigherTolerance(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "diameter.tolerance.upper".to_string(),
            value: ScalarValue::Float(v),
        },
        Mutation::SetGlobalWarning(v) => RuntimeRequestKind::SetConfigProperty {
            target: ident,
            path: "out_of_tolerance.active".to_string(),
            value: ScalarValue::Boolean(v),
        },
    })
}

fn init_state_event(instance: &MachineInstance) -> serde_json::Value {
    _ = instance;
    serde_json::json!({})
}

fn init_measurements_event(instance: &MachineInstance) -> serde_json::Value {
    let get = |name: &'static str| -> Option<f64> {
        instance
            .measurements
            .get(name)
            .expect("Missing property")
            .as_ref()
            .expect("Property was not initialized")
            .value
    };

    serde_json::json!({
        "diameter":   get("diameter").expect("Non nullable measurement is null"),
        "x_diameter": get("diameter_x"),
        "y_diameter": get("diameter_y"),
        "roundness":  get("roundness"),
    })
}

// --- live values event ---
#[derive(Serialize)]
struct LiveValuesEvent {
    pub diameter: f64,
    pub x_diameter: Option<f64>,
    pub y_diameter: Option<f64>,
    pub roundness: Option<f64>,
}

// --- state event ---
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

// --- mutation ---
#[derive(Deserialize)]
#[allow(clippy::enum_variant_names)]
enum Mutation {
    SetTargetDiameter(f64),
    SetLowerTolerance(f64),
    SetHigherTolerance(f64),
    SetGlobalWarning(bool),
}
