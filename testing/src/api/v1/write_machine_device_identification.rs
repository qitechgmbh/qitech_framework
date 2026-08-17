use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::State;
use axum::http::Response;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use qitech_framework_core::ident::DeviceMachineIdentification;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_hub::ModuleContext;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Request {
    pub ident_device: DeviceMachineIdentification,
    pub ident_hardware: DeviceHardwareIdentificationEthercat,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}

pub async fn post(
    State(ctx): State<Arc<ModuleContext>>,
    Json(body): Json<Request>,
) -> Response<Body> {
    ctx.request_tx
        .send(RuntimeRequest {
            request_id: 0,
            kind: RuntimeRequestKind::WriteMachineDeviceInfo {
                machine_ident: body.ident_device.machine_ident,
                role: body.ident_device.role,
                subdevice_index: body.ident_hardware.subdevice_index,
            },
        })
        .await
        .unwrap();

    (StatusCode::OK, ()).into_response()
}
