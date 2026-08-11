use std::thread;
use std::time::Duration;

use qitech_framework::runtime::Runtime;
use qitech_framework::runtime::RuntimeConfiguration;
use qitech_framework::session;
use qitech_framework_tui::Tui;
use qitech_framework_tui::TuiConfiguration;

pub mod beckhoff_el2004_machine;
pub mod laser_v1;

pub fn run_headless(config: RuntimeConfiguration) {
    let session = session::debug::runtime();
    let rt = Runtime::init(config, session).unwrap();
    rt.run().unwrap();
}

pub fn run_tui(config: RuntimeConfiguration) {
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
