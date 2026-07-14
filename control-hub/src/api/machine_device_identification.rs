#[derive(serde::Deserialize, Debug)]
pub struct MachineDeviceInfoRequest {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

pub async fn post(
    State(app_state): State<Arc<SharedAppState>>,
    Json(body): Json<MachineDeviceInfoRequest>,
) -> Response<axum::body::Body> {
    // We only save the mapping.
    // The front-end will ask the user to restart.

    let dev_addr = body.hardware_identification_ethercat.subdevice_index as u16;

    let mut idents = match persist::read_machine_device_info() {
        Ok(i) => i,
        Err(e) => return ResponseUtil::error(&e.to_string()),
    };

    let mut ident = idents.iter_mut().find(|i| i.device_address == dev_addr);

    if let Some(ident) = ident.as_mut() {
        ident.role = body.device_machine_identification.role;
        ident.machine_vendor = body
            .device_machine_identification
            .machine_identification_unique
            .machine_identification
            .vendor;
        ident.machine_id = body
            .device_machine_identification
            .machine_identification_unique
            .machine_identification
            .machine;
        ident.machine_serial = body
            .device_machine_identification
            .machine_identification_unique
            .serial;
    } else {
        idents.push(MachineDeviceInfo {
            role: body.device_machine_identification.role,
            machine_id: body
                .device_machine_identification
                .machine_identification_unique
                .machine_identification
                .machine,
            machine_vendor: body
                .device_machine_identification
                .machine_identification_unique
                .machine_identification
                .vendor,
            machine_serial: body
                .device_machine_identification
                .machine_identification_unique
                .serial,
            device_address: dev_addr,
        });
    }

    // json
    // if let Err(e) = persist::write_machine_device_info(&idents) {
    //     return ResponseUtil::error(&e.to_string())
    // }

    // legacy eeprom
    if let Some(channel) = &app_state.ethercat_thread_channel {
        if let Err(e) = channel.write_machine_device_info_eeprom(idents.clone()) {
            return ResponseUtil::error(&e.to_string());
        }
    } else {
        println!("Tried write_machine_device_info without an EtherCatChannel?")
    }

    ResponseUtil::ok(MutationResponse::success())
}
