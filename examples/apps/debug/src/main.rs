use qitech_framework::HubConfiguration;
use qitech_framework::run_with_hub;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

mod machine;
use machine::LaserV1;

mod api;
use api::ApiServer;
use api::SharedState;
use api::SocketIODispatcher;

use crate::api::LegacySharedState;

#[tokio::main]
pub async fn main() {
    // --- init tracing subscriber ---
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(true)
        // .with_max_level(tracing::Level::DEBUG)
        .init();

    // --- configure runtime ---
    let config_rt = RuntimeConfiguration::new()
        .ethercat(EtherCATConfig::default())
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.0-usbv2-0:2.3:1.0-port0".to_string(),
            LaserV1::IDENTIFICATION.unique(1),
            1,
            None,
        )
        .machine::<LaserV1>();

    // --- configure hub ---
    let state = SharedState::default();
    let state_legacy = LegacySharedState::new();

    let config_hub = HubConfiguration::new()
        .listener(SocketIODispatcher::new(state.clone(), state_legacy.clone()))
        .actor(ApiServer::new(state, state_legacy));

    // --- run ---
    run_with_hub(config_rt, config_hub).await.unwrap();
}
