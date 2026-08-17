use std::time::Duration;

use qitech_framework::run_with_hub;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_framework_hub::HubConfiguration;
use qitech_framework_hub::Module;
use qitech_framework_hub::ModuleContext;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

mod laser_v1;
use laser_v1::LaserV1;

mod api;

#[tokio::main]
pub async fn main() {
    const LASER_SLAVE_ID: u8 = 1;

    // --- configure runtime ---
    let runtime_config = RuntimeConfiguration::new()
        .cycle_period(Duration::from_millis(100))
        .export_interval(Duration::from_secs(1))
        .modbus_rtu_device::<LaserDevice>(
            "pci-0000:c6:00.3-usbv2-0:1.4:1.0-port0",
            LaserV1::IDENTIFICATION.unique(1),
            LASER_SLAVE_ID,
            None,
        )
        .machine::<LaserV1>();

    // --- configure hub ---
    let hub_config = HubConfiguration::new().module(PrintModule);

    // --- run ---
    run_with_hub(runtime_config, hub_config).await.unwrap();
}

struct PrintModule;

impl Module for PrintModule {
    async fn run(self, mut ctx: ModuleContext) {
        loop {
            // ctx.request_tx.send(value);

            let report = ctx.report_rx.recv().await.unwrap();
            println!("Received report");
        }
    }
}

/*
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
*/
