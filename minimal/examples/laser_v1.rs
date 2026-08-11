use std::time::Duration;

use minimal::{laser_v1::LaserV1, run_tui};
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

pub fn main() {
    // --- configure runtime ---
    let config = RuntimeConfiguration::new()
        .cycle_period(Duration::from_millis(100))
        // .export_interval(Duration::from_secs(2))
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0",
            LaserV1::IDENTIFICATION.into_unique(1),
            1,
            None,
        )
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.3-usbv2-0:1.4:1.0-port0",
            LaserV1::IDENTIFICATION.into_unique(2),
            1,
            None,
        )
        .machine::<LaserV1>();

    // --- run it ---
    run_tui(config)
    // run_headless(config)
}
