use qitech_framework::MachineIdentificationUnique;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::request::RuntimeRequestKind;
use serde::Deserialize;

// --- mutation ---
#[derive(Deserialize)]
#[allow(clippy::enum_variant_names)]
enum Mutation {
    SetTargetDiameter(f64),
    SetLowerTolerance(f64),
    SetHigherTolerance(f64),
    SetGlobalWarning(bool),
}

pub fn map_request(
    ident: MachineIdentificationUnique,
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

// --- live values event ---

// --- state event ---
