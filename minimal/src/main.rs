use std::thread;
use std::time::Duration;

use qitech_framework::runtime::Runtime;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_framework::session;
use qitech_framework_tui::Tui;
use qitech_framework_tui::TuiConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

mod laser_v1;
use laser_v1::LaserV1;

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
}

fn run_headless(config: RuntimeConfiguration) {
    let session = session::debug::runtime();
    let rt = Runtime::init(config, session).unwrap();
    rt.run().unwrap();
}

fn run_tui(config: RuntimeConfiguration) {
    let (session_rt, session_tui) = session::crossbeam(64);

    thread::spawn(move || {
        let rt = Runtime::init(config, session_rt).unwrap();
        _ = rt.run();
    });

    // run slightly faster than the export interval so we don't stay behind
    let config = TuiConfiguration::new().refresh_rate(Duration::from_secs_f64(1.0 / 40.0));

    // TODO: remove anyhow from TUI
    let app = Tui::create(config).unwrap();
    app.run(session_tui).unwrap()
}
