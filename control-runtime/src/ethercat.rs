use std::{
    cell::RefCell,
    rc::Rc,
    sync::Arc,
    thread::{self, sleep},
    time::{Duration, Instant},
};
use anyhow::bail;

use qitech_lib::ethercat_hal::{
    BECKHOFF_VENDOR_ID, EtherCATControl, EtherCATState, Mailbox, MasterConfiguration, MetaSubdevice, TripleBufConsumer, init_ethercat
};
use qitech_lib::ethercat_hal::devices::{
    EthercatDevice, device_from_subdevice_identity_rc
};
use qitech_lib::ethercat_hal::interface_discovery::{
    LinkType, list_ethernet_interfaces, test_interface
};

use crate::machine::{MachineHardwareRegistry, hardware};

pub type Controller = EtherCATControl<TripleBufConsumer, Arc<Mailbox>>;
pub type Device = Rc<RefCell<dyn EthercatDevice + 'static>>;

pub fn find_interface(retry_delay: Duration) -> String {
    loop {
        let interfaces = match list_ethernet_interfaces() {
            Ok(interfaces) => interfaces,
            Err(err) => {
                println!(
                    "Could not list ethernet interfaces ({err:?}), retrying in {retry_delay:?}..."
                );
                thread::sleep(retry_delay);
                continue;
            }
        };

        let names = interfaces
            .iter()
            .map(|interface| interface.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        println!("testing interfaces: [{}]", names);

        for interface in interfaces {
            if !matches!(interface.link_type, LinkType::Link) {
                continue;
            }

            if test_interface(&interface.name).is_ok() {
                return interface.name;
            }
        }

        println!("no interface found, retrying in {retry_delay:?}...");
        thread::sleep(retry_delay);
    }
}

pub fn init(interface: &str, config: Option<MasterConfiguration>) -> Controller {
    init_ethercat(interface, config)
}

pub fn setup(
    controller: &Controller,
    hardware_registry: &mut MachineHardwareRegistry,
) -> Result<Vec<(MetaSubdevice, Device)>, anyhow::Error> {
    // switch into pre op mode
    controller.channel.request_state_change(EtherCATState::PreOp)?;

    // Require 2 consecutive stable polls (~100 ms) in PreOp before proceeding.
    // One poll is not enough: the state machine may still be mid-iteration on first observation,
    // causing EEPROM reads to contend with its ongoing preop_group tick.
    let deadline = Instant::now() + Duration::from_secs(10);

    let mut stable_ticks: u32 = 0;
    while stable_ticks < 2 {
        if is_controller_finished(controller) || Instant::now() >= deadline {
            bail!("No response from state machine or Timeout");
        }

        sleep(Duration::from_millis(50));

        let preop_ready = 
            controller.app_handle.get_state() == EtherCATState::PreOp
            && controller.app_handle.get_subdevice_count() > 0;

        if preop_ready {
            stable_ticks += 1
        } else {
            stable_ticks = 0
        }
    }

    let mut idents = vec![];

    println!(
        "Initialized {} subdevices",
        controller.app_handle.get_subdevice_count()
    );

    let mut subdevices = Vec::new();
    for meta in controller.app_handle.try_get_subdevices_vec_sync()? {
        let dev = match device_from_subdevice_identity_rc(&meta) {
            Ok(d) => d,
            Err(_) => {
                println!(
                    "No EtherCAT device implementation for {:?}",
                    meta.get_name()
                );
                continue;
            }
        };

        println!("pushing: {meta:?}");

        subdevices.push((meta, dev.clone()));
        if meta.vendor == BECKHOFF_VENDOR_ID {
            controller
                .channel
                .set_mut_beckhoff_eeprom_lock_active(meta.device_address)?;
        }
    }

    match controller.channel.read_device_identifications() {
        Ok(mut eeprom_idents) => {
            hardware::append_ethercat(
                hardware_registry, 
                &eeprom_idents,
                &subdevices,
            );

            idents.append(&mut eeprom_idents);
        }
        Err(e) => {
            println!("Could not read device identifications from eeprom: {:?}", e);
        }
    };

    // TODO: find way to emit this later
    // let _res = state.fill_ethercat_metadata(eth_control, idents);

    Ok(subdevices)
}

pub fn finalize(
    controller: &Controller,
    sub_devices: &mut Vec<(MetaSubdevice, Device)>,
) -> Result<(), anyhow::Error> {
    // go into op mode
    controller.channel.request_state_change(EtherCATState::Op)?;
    wait_for_op_state(controller)?;

    // update offsets in sub devices
    let src = controller.app_handle.try_get_subdevices_vec_sync()?;
    for (meta, _) in sub_devices {
        let Some(src) = find_subdevice(&src, meta.device_address) else {
            panic!("EtherCAT device suddenly missing in finalize_ethercat");
        };

        update_sub_device_offsets(meta, src);
    }

    Ok(())
}

// --- utils ---
fn wait_for_op_state(controller: &Controller) -> Result<(), anyhow::Error> {
    while !controller.app_handle.check_all_op() {
        if is_controller_finished(controller) {
            bail!("Failed to reach OP State");
        }

        sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn find_subdevice(
    sub_devices: &[MetaSubdevice], 
    device_address: u16
) -> Option<&MetaSubdevice> {
        sub_devices
        .iter()
        .find(|&device| device.device_address == device_address)
        .map(|v| v as _)
}

fn update_sub_device_offsets(dest: &mut MetaSubdevice, src: &MetaSubdevice) {
    dest.start_tx = src.start_tx;
    dest.end_tx = src.end_tx;
    dest.start_rx = src.start_rx;
    dest.end_rx = src.end_rx;
}

pub fn is_controller_finished(controller: &Controller) -> bool {
    match &controller.join_handle {
        Some(handle) => handle.is_finished(),
        None => false,
    }
}
