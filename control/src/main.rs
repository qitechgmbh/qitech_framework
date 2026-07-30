use std::thread;
use std::time::Duration;

use qitech_framework::MachineIdentificationUnique;
use qitech_framework::Runtime;
use qitech_framework::machine::MachineInterface;
use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeConfiguration;

mod types;
mod utils;

mod controllers;
mod converters;
mod interface;

mod machines;
use machines::LaserV1;
use machines::Winder2;
use qitech_framework::runtime::bridge::MockBridge;
use qitech_framework::runtime::bridge::crossbeam::CrossbeamBridge;
use qitech_framework::runtime::bridge::crossbeam::CrossbeamBridgeBootstrap;
use qitech_lib::ethercat_hal::DcConfiguration;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::ethercat_hal::MasterTxRxConfig;
use qitech_lib::ethercat_hal::RtOptimizationConfig;

// udevadm info --query=property --name=/dev/ttyUSB0 | grep ID_PATH_TAG

pub fn main() -> anyhow::Result<()> {
    interface::bring_up_all_ethernet();

    let laser_ident = |serial: u16| MachineIdentificationUnique {
        identification: LaserV1::IDENTIFICATION,
        serial,
    };

    // --- configure runtime ---
    let config = RuntimeConfiguration::new()
        .requests_per_cycle_max(10)
        .export_interval(Duration::from_secs_f64(1.0 / 4.0))
        .ethercat(ETHERCAT_CONFIG)
        .modbus_rtu_device("pci-0000:c6:00.0-usbv2-0:2.3:1.0-port0", laser_ident(1))
        .modbus_rtu_device("pci-0000:c6:00.0-usbv2-0:2.1:1.0-port0", laser_ident(2))
        .machine::<LaserV1>()
        .machine::<Winder2>();

    run_tui(config)
}

fn run_tui(config: RuntimeConfiguration) -> anyhow::Result<()> {
    let (bridge, handle) = CrossbeamBridgeBootstrap::new();

    // --- start runtime in new thread ---
    thread::spawn(move || {
        let rt = Runtime::<CrossbeamBridge>::init(config, bridge).unwrap();
        rt.run();
    });

    // --- start tui in main thread ---
    let schemas = vec![LaserV1::SCHEMA, Winder2::SCHEMA];
    qitech_framework_tui::run(schemas, handle)
}

fn run_cli(config: RuntimeConfiguration) -> anyhow::Result<()> {
    let rt = Runtime::<MockBridge>::init(config, MockBridge).unwrap();
    rt.run();
    Ok(())
}

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
