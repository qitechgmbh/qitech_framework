use std::time::Duration;

use minimal::{beckhoff_el2004_machine::EL2004Machine, run_headless, run_tui};
use qitech_framework::runtime::{EtherCATConfig, RuntimeConfiguration};

pub fn main() {
    // --- configure runtime ---
    let config = RuntimeConfiguration::new()
        .cycle_period(Duration::from_millis(100))
        .ethercat(EtherCATConfig {
            interface_scan_interval: Duration::from_secs(1),
            master_config: None,
            stay_in_preop: false
        })
        .machine::<EL2004Machine>();

    // --- run it ---
    run_tui(config)
    // run_headless(config)
}

