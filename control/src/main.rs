use std::thread;
use std::time::Duration;

use qitech_framework::MachineIdentificationUnique;
use qitech_framework::Runtime;
use qitech_framework::machine::MachineInterface;
use qitech_framework::runtime::RuntimeConfiguration;

mod types;
mod utils;

mod controllers;
mod converters;
mod interface;

mod machines;
use machines::LaserV1;
use machines::WinderV1;
use qitech_framework::runtime::bridge::MockBridge;
use qitech_framework::runtime::bridge::crossbeam::CrossbeamBridge;
use qitech_framework::runtime::bridge::crossbeam::CrossbeamBridgeBootstrap;

// udevadm info --query=property --name=/dev/ttyUSB0 | grep ID_PATH_TAG

pub fn main() -> anyhow::Result<()> {
    interface::bring_up_all_ethernet();

    let laser = MachineIdentificationUnique {
        identification: LaserV1::IDENTIFICATION,
        serial: 1,
    };

    let config = RuntimeConfiguration::default()
        // .ethercat(ETHERCAT_CONFIG)
        .export_interval(Duration::from_secs_f64(0.25))
        .modbus_rtu_device("pci-0000_c6_00_0-usb-0_2_1_1_0", laser)
        .machine::<LaserV1>()
        .machine::<WinderV1>();

    let (bridge, handle) = CrossbeamBridgeBootstrap::new();

    // --- start runtime in new thread ---
    thread::spawn(move || {
        let rt = Runtime::<CrossbeamBridge>::init(config, bridge).unwrap();
        rt.run();
    });

    // --- start tui in main thread ---
    let schemas = vec![
        LaserV1::SCHEMA,
    ];

    qitech_framework_tui::run(schemas, handle)
}

/*
const ETHERCAT_CONFIG: EtherCATConfig = {
    let target_cycle_time_us: u64 = 1000;

    let dc_config = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 500,
    };

    let opt_config = RtOptimizationConfig {
        ethercat_loop_thread_core: 3,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 50,
        pin_irq_core: Some(3),
        lock_memory: true,
    };

    let master_config = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
    };

    EtherCATConfig {
        interface_scan_interval: Duration::from_secs(2),
        master_config: Some(master_config),
        stay_in_preop: false,
    }
};
*/