use std::thread;
use std::time::Duration;

use qitech_framework::MachineIdentificationUnique;
use qitech_framework::Runtime;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_framework::session;
use qitech_framework_tui::Tui;
use qitech_framework_tui::TuiConfiguration;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

mod laser_v1;
use laser_v1::LaserV1;

pub fn main() -> anyhow::Result<()> {
    let laser_ident = |serial: u16| MachineIdentificationUnique {
        identification: LaserV1::IDENTIFICATION,
        serial,
    };

    // --- configure runtime ---
    let config = RuntimeConfiguration::new()
        .cycle_timeout(Duration::from_millis(100))
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0",
            laser_ident(1),
            1,
            None,
        )
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.3-usbv2-0:1.4:1.0-port0",
            laser_ident(2),
            1,
            None,
        )
        .machine::<LaserV1>();

    run_tui(config)
}

fn run_tui(config: RuntimeConfiguration) -> anyhow::Result<()> {
    let (session_rt, session_tui) = session::crossbeam(64);

    thread::spawn(move || {
        let rt = Runtime::init(config, session_rt).unwrap();
        rt.run();
    });

    // run slightly faster than the export interval so we don't stay behind
    let config = TuiConfiguration::new().refresh_rate(Duration::from_secs_f64(1.0 / 40.0));

    let app = Tui::create(config)?;
    app.run(session_tui)
}
