use std::time::Duration;

use qitech_framework::runtime::EtherCATConfig;
use qitech_framework::runtime::RuntimeBuilder;
use qitech_framework::runtime::bridge::MockBridge;
use qitech_lib::ethercat_hal::DcConfiguration;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::ethercat_hal::MasterTxRxConfig;
use qitech_lib::ethercat_hal::RtOptimizationConfig;

mod types;
mod utils;

mod controllers;
mod converters;
mod interface;

mod machines;
use machines::LaserV1;
use machines::WinderV1;

pub fn main() -> anyhow::Result<()> {
    interface::bring_up_all_ethernet();

    let rt = RuntimeBuilder::new()
        // .ethercat(ETHERCAT_CONFIG)
        .machine::<LaserV1>()
        .machine::<WinderV1>()
        .build(MockBridge)?;

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
