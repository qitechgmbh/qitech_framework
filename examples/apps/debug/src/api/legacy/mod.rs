use std::collections::HashMap;

use qitech_framework::MachineIdentification;

mod types;
use types::EtherCATDeviceMetadata;
use types::MachineIdentificationUnique;

mod socketio;
use socketio::MachineNamespaceManager;
use socketio::MainNamespaceManager;
pub use socketio::SocketIODispatcher;
pub use socketio::init as init_socket_io;

use crate::api::Swappable;

mod adapter;
use adapter::MachineLegacyDataAdapter;

#[derive(Clone)]
pub struct LegacySharedState {
    ns_main: Swappable<MainNamespaceManager>,
    ns_machines: Swappable<MachineNamespaceManager>,
    adapters: HashMap<MachineIdentification, MachineLegacyDataAdapter>,
}

impl LegacySharedState {
    pub fn new() -> Self {
        let mut adapters = HashMap::new();

        adapters.insert(
            MachineIdentification {
                vendor_id: 1,
                machine_id: 6,
            },
            adapter::LASER_V1,
        );

        Self {
            ns_main: Default::default(),
            ns_machines: Default::default(),
            adapters,
        }
    }
}
