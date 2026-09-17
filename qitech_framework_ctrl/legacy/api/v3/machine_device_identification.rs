use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MachineDeviceInfoRequest {
    pub device_machine_identification: DeviceMachineIdentification,
    pub hardware_identification_ethercat: DeviceHardwareIdentificationEthercat,
}

pub fn post() {

}