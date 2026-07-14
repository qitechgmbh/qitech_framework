use std::{sync::Arc, thread, time::Duration};

use anyhow::bail;
use qitech_lib::ethercat_hal::{self, BECKHOFF_VENDOR_ID, DcConfiguration, EtherCATControl, Mailbox, MasterConfiguration, RtOptimizationConfig, TripleBufConsumer, devices::device_from_subdevice_identity_rc, interface_discovery::{LinkType, list_ethernet_interfaces, test_interface}};

pub fn find_interface(retry_delay: Duration) -> String {
    loop {
        let interfaces = match list_ethernet_interfaces() {
            Ok(interfaces) => interfaces,
            Err(err) => {
                println!("Could not list ethernet interfaces ({err:?}), retrying in {retry_delay:?}...");
                thread::sleep(retry_delay);
                continue;
            }
        };

        for interface in interfaces {
            if matches!(interface.link_type, LinkType::Link) {
                continue;
            }

            if test_interface(&interface.name).is_ok() {
                println!("{} is EtherCAT", interface.name);
                return interface.name;
            }

            println!("{} is not EtherCAT", interface.name);
        }

        println!("No EtherCAT interface found, retrying in {retry_delay:?}...");
        thread::sleep(retry_delay);
    }
}

pub fn init(interface: &str) -> EtherCATControl<TripleBufConsumer, Arc<Mailbox>> {
    let target_cycle_time_us: u64 = 1000;
    let dc_config: DcConfiguration = DcConfiguration {
        start_delay: Duration::from_millis(100),
        sync0_period: Duration::from_micros(target_cycle_time_us),
        sync0_shift: Duration::from_micros(target_cycle_time_us / 2),
        target_dc_tick: 500,
    };

    let opt_config: RtOptimizationConfig = RtOptimizationConfig {
        ethercat_loop_thread_core: 3,
        ethercat_loop_thread_priority: 99,
        ethercat_io_thread_core: 3,
        ethercat_io_thread_priority: 50,
        pin_irq_core: Some(3),
        lock_memory: true,
    };

    let config: MasterConfiguration = MasterConfiguration {
        target_cycle_time_us: target_cycle_time_us as usize,
        tx_rx_config: ethercat_hal::MasterTxRxConfig::TxRxIoUring,
        realtime_optimizations: Some(opt_config),
        dc_config,
        wkc_mismatch_threshold: 5,
        op_ramp_grace_cycles: 10000,
    };
    
    ethercat_hal::init_ethercat(interface, Some(config))
}

pub fn setup(
    // state: Arc<SharedAppState>,
    // main_state: &mut MainState,
    eth_control: &EtherCATControl<TripleBufConsumer, Arc<Mailbox>>,
) -> Result<(), anyhow::Error> {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);

    // Require 2 consecutive stable polls (~100 ms) in PreOp before proceeding.
    // One poll is not enough: the state machine may still be mid-iteration on first observation,
    // causing EEPROM reads to contend with its ongoing preop_group tick.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let mut stable_ticks: u32 = 0;
    while stable_ticks < 2 {
        // State-machine thread died, or timeout — bail for a clean restart.
        if eth_control
            .join_handle
            .as_ref()
            .map_or(false, |h| h.is_finished())
            || std::time::Instant::now() >= deadline
        {
            bail!("No response from state machine Timeout");
        }
        std::thread::sleep(Duration::from_millis(50));

        let preop_ready = eth_control.app_handle.get_state()
            == qitech_lib::ethercat_hal::EtherCATState::PreOp
            && eth_control.app_handle.get_subdevice_count() > 0;

        if preop_ready {
            stable_ticks += 1
        } else {
            stable_ticks = 0
        }
    }

    let mut idents = vec![];
    println!(
        "Initialized {} subdevices",
        eth_control.app_handle.get_subdevice_count()
    );

    for meta in eth_control.app_handle.try_get_subdevices_vec_sync()? {
        let dev = device_from_subdevice_identity_rc(&meta);

        let dev = match dev {
            Ok(d) => d,
            Err(_) => {
                println!("Ecat {:?} is not implemented", meta.get_name());
                continue;
            }
        };

        main_state.subdevices.push((meta.clone(), dev.clone()));
        if meta.vendor == BECKHOFF_VENDOR_ID {
            let _res = eth_control
                .channel
                .set_mut_beckhoff_eeprom_lock_active(meta.device_address);
        }
    }

    match eth_control.channel.read_device_identifications() {
        Ok(mut eeprom_idents) => {
            main_state.generate_machine_hardware_from_ethercat(
                &eeprom_idents,
                main_state.subdevices.clone(),
                eth_control.channel.clone(),
            );
            idents.append(&mut eeprom_idents);
        }
        Err(e) => {
            println!("Could not read device identifications from eeprom: {:?}", e);
        }
    };
    
    let _res = state.fill_ethercat_metadata(eth_control, idents);

    Ok(())
}

pub fn finalize(
    // main_state: &mut MainState,
    eth_control: &EtherCATControl<TripleBufConsumer, Arc<Mailbox>>,
) -> Result<(), anyhow::Error> {
    let _res = eth_control
        .channel
        .request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);

    while !eth_control.app_handle.check_all_op() {
        if eth_control
            .join_handle
            .as_ref()
            .is_some_and(|h| h.is_finished())
        {
            // State machine died before reaching OP — bail so main_logic can exit cleanly.
            bail!("Failed to reach OP State!");
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let subdevices = eth_control.app_handle.try_get_subdevices_vec_sync()?;
    _ = subdevices;

    // for meta in &mut main_state.subdevices {
    //     let m = subdevices
    //         .iter()
    //         .find(|m| m.device_address == meta.0.device_address)
    //         .expect("Ethercat Device Suddenly Missing in finalize_ethercat");
    // 
    //     meta.0.start_tx = m.start_tx;
    //     meta.0.end_tx = m.end_tx;
    //     meta.0.start_rx = m.start_rx;
    //     meta.0.end_rx = m.end_rx;
    // }
    
    Ok(())
}