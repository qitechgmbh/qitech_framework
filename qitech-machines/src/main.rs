use std::time::Duration;
use qitech_lib::ethercat_hal::{
    DcConfiguration, MasterConfiguration, 
    MasterTxRxConfig, RtOptimizationConfig
};
use control_runtime::{Config, MachineRegistry, Runtime};

pub fn main() -> anyhow::Result<()> {
    let mut registry = MachineRegistry::default();

    // register winder 
    let schema = include_str!("../../schemas/winder_v1.yaml");
    registry.register::<WinderV1>(schema)?;

    // create runtime
    let runtime = Runtime::init(get_config(), registry)?;

    // start runtime
    runtime.run()
}

pub fn get_config() -> Config {
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

    let ethercat = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
    };

    Config {
        interface_discovery_retry_interval: Duration::from_secs(2),
        ethercat: Some(ethercat),
    }
}
