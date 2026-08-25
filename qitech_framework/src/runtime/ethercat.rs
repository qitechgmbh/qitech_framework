use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::ident::DeviceHardwareIdentification;
use qitech_framework_core::ident::DeviceHardwareIdentificationEthercat;
use qitech_framework_core::ident::DeviceIdentification;
use qitech_framework_core::ident::DeviceMachineAssignment;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::EtherCATDeviceMetadata;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionInitializing;
use qitech_lib::ethercat_hal;
use qitech_lib::ethercat_hal::BECKHOFF_VENDOR_ID;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::devices::device_from_subdevice_identity_rc;
use qitech_lib::ethercat_hal::interface_discovery::LinkType;
use qitech_lib::ethercat_hal::interface_discovery::list_ethernet_interfaces;
use qitech_lib::ethercat_hal::interface_discovery::test_interface;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;

use super::error::EtherCATInitializeError;
use super::error::EtherCATInitializeResult;
use super::error::RuntimeInitializeError;
use super::error::RuntimeInitializeResult;
use crate::machine::Hardware;
use crate::machine::hardware::EtherCATDeviceIdentified;
use crate::runtime::EtherCATConfig;
use crate::runtime::EtherCATController;
use crate::runtime::EtherCATSubDevice;
use crate::runtime::types::HardwareRegistry;

#[tracing::instrument(skip_all)]
pub fn init<T: RuntimeTransport>(
    config: EtherCATConfig,
    session: &mut SessionInitializing<T>,
    hardware_registry: &mut HardwareRegistry,
) -> RuntimeInitializeResult<(Option<EtherCATController>, Vec<EtherCATSubDevice>)> {
    session.send_event(RuntimeInitEvent::EtherCATDiscoveryStarted)?;

    let interface = find_interface(config.interface_scan_interval);

    session.send_event(RuntimeInitEvent::EtherCATDiscoveryCompleted {
        interface: interface.clone(),
    })?;

    let controller = ethercat_hal::init_ethercat(&interface, Some(config.master_config));

    let state = match controller.app_handle.get_state() {
        ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
        ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
        ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
        ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
        ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
        ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
    };

    session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;
    session.send_event(RuntimeInitEvent::EtherCATInitializationStarted)?;
    let sub_devices = setup(&controller)?;

    let state = match controller.app_handle.get_state() {
        ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
        ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
        ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
        ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
        ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
        ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
    };
    session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;

    let devices = read_and_register_identifications(&controller, &sub_devices, hardware_registry);

    session.send_event(RuntimeInitEvent::EtherCATDeviceInitializationCompleted {
        devices: build_ecat_metadata(&sub_devices, &devices),
    })?;

    Ok((Some(controller), sub_devices))
}

#[tracing::instrument]
pub fn find_interface(retry_delay: Duration) -> String {
    loop {
        let interfaces = match list_ethernet_interfaces() {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(
                    ?err,
                    ?retry_delay,
                    "could not list ethernet interfaces, retrying"
                );

                thread::sleep(retry_delay);
                continue;
            }
        };

        for interface in interfaces {
            tracing::debug!(interface = %interface.name, "testing interface");

            if !matches!(interface.link_type, LinkType::Link) {
                continue;
            }

            if test_interface(&interface.name).is_ok() {
                return interface.name;
            }
        }

        tracing::warn!(?retry_delay, "no interface found, retrying");
        thread::sleep(retry_delay);
    }
}

#[tracing::instrument(skip_all)]
pub fn setup(controller: &EtherCATController) -> RuntimeInitializeResult<Vec<EtherCATSubDevice>> {
    // switch into pre op mode
    controller
        .channel
        .request_state_change(ethercat_hal::EtherCATState::PreOp)
        .map_err(EtherCATInitializeError::FailedToRequestStateChange)?;

    // Require 2 consecutive stable polls (~100 ms) in PreOp before proceeding.
    // One poll is not enough: the state machine may still be mid-iteration on first observation,
    // causing EEPROM reads to contend with its ongoing preop_group tick.
    let deadline = Instant::now() + Duration::from_secs(10);

    let mut stable_ticks: u32 = 0;
    while stable_ticks < 2 {
        if is_controller_finished(controller) || Instant::now() >= deadline {
            return Err(EtherCATInitializeError::NoResponseFromStateMachineOrTimeout.into());
        }

        thread::sleep(Duration::from_millis(50));

        let preop_ready = controller.app_handle.get_state() == ethercat_hal::EtherCATState::PreOp
            && controller.app_handle.get_subdevice_count() > 0;

        if preop_ready {
            stable_ticks += 1
        } else {
            stable_ticks = 0
        }
    }

    tracing::info!(
        count = controller.app_handle.get_subdevice_count(),
        "initialized subdevices"
    );

    let meta_subdevices = controller
        .app_handle
        .try_get_subdevices_vec_sync()
        .map_err(EtherCATInitializeError::FailedToGetSubDevices)?;

    let mut subdevices = Vec::new();

    for meta in meta_subdevices {
        let dev = match device_from_subdevice_identity_rc(&meta) {
            Ok(d) => d,
            Err(_) => {
                tracing::warn!(name = ?meta.get_name(), "no EtherCAT device implementation");
                continue;
            }
        };

        subdevices.push((meta, dev.clone()));
        if meta.vendor == BECKHOFF_VENDOR_ID {
            controller
                .channel
                .set_mut_beckhoff_eeprom_lock_active(meta.device_address)
                .map_err(EtherCATInitializeError::FailedToSetBeckhoffEepromLockActive)?;
        }
    }

    Ok(subdevices)
}

#[tracing::instrument(skip_all)]
pub fn finalize(
    controller: &EtherCATController,
    sub_devices: &mut Vec<EtherCATSubDevice>,
) -> RuntimeInitializeResult<()> {
    // go into op mode
    controller
        .channel
        .request_state_change(ethercat_hal::EtherCATState::Op)
        .map_err(EtherCATInitializeError::FailedToRequestStateChange)?;

    wait_for_op_state(controller)?;

    // update offsets in sub devices
    let src = controller
        .app_handle
        .try_get_subdevices_vec_sync()
        .map_err(EtherCATInitializeError::FailedToGetSubDevices)?;

    for (meta, _) in sub_devices {
        let Some(src) = find_subdevice(&src, meta.device_address) else {
            return Err(RuntimeInitializeError::AssertionFailed(
                "EtherCAT device suddenly missing in finalize_ethercat",
            ));
        };

        update_sub_device_offsets(meta, src);
    }

    Ok(())
}

// --- utils ---
pub fn is_controller_finished(controller: &EtherCATController) -> bool {
    match &controller.join_handle {
        Some(handle) => handle.is_finished(),
        None => false,
    }
}

fn wait_for_op_state(controller: &EtherCATController) -> EtherCATInitializeResult<()> {
    while !controller.app_handle.check_all_op() {
        if is_controller_finished(controller) {
            return Err(EtherCATInitializeError::FailedToReachOpState);
        }

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn find_subdevice(sub_devices: &[MetaSubdevice], device_address: u16) -> Option<&MetaSubdevice> {
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

// --- deconstructing ---
fn read_and_register_identifications(
    controller: &EtherCATController,
    subdevices: &[EtherCATSubDevice],
    hardware_registry: &mut HardwareRegistry,
) -> Vec<MachineDeviceInfo> {
    let mut idents = Vec::new();

    match controller.channel.read_device_identifications() {
        Ok(mut eeprom_idents) => {
            append_ethercat(hardware_registry, &eeprom_idents, subdevices);
            idents.append(&mut eeprom_idents);
        }
        Err(err) => {
            tracing::error!(?err, "could not read device identifications from eeprom");
        }
    }

    idents
}

fn build_ecat_metadata(
    subdevices: &[EtherCATSubDevice],
    idents: &[MachineDeviceInfo],
) -> Vec<EtherCATDeviceMetadata> {
    subdevices
        .iter()
        .map(|(meta, _)| {
            let device_machine_identification = idents
                .iter()
                .find(|info| info.device_address == meta.device_address)
                .map(|info| DeviceMachineAssignment {
                    machine: MachineInstanceIdentification {
                        machine: MachineIdentification {
                            vendor_id: info.machine_vendor,
                            machine_id: info.machine_id,
                        },
                        serial: info.machine_serial,
                    },
                    role: info.role,
                });

            EtherCATDeviceMetadata {
                configured_address: meta.device_address,
                name: meta.get_name().expect("please be a utf-8"),
                vendor_id: meta.vendor,
                product_id: meta.product_id,
                revision: meta.revision,
                device_identification: DeviceIdentification {
                    assignment: device_machine_identification,
                    hardware: DeviceHardwareIdentification::Ethercat(
                        DeviceHardwareIdentificationEthercat {
                            subdevice_index: meta.device_address as usize,
                        },
                    ),
                },
            }
        })
        .collect()
}

fn append_ethercat(
    hardware_registry: &mut HardwareRegistry,
    device_infos: &[MachineDeviceInfo],
    mapped_ecat_devices: &[EtherCATSubDevice],
) {
    let combined_list = create_mapped_ethercat_devices(device_infos, mapped_ecat_devices);

    for (info, device) in combined_list {
        let identification = MachineInstanceIdentification {
            serial: info.machine_serial,
            machine: MachineIdentification {
                vendor_id: info.machine_vendor,
                machine_id: info.machine_id,
            },
        };

        hardware_registry
            .entry(identification)
            .or_default()
            .push(Hardware::Ethercat(EtherCATDeviceIdentified {
                device,
                info,
            }));
    }
}

fn create_mapped_ethercat_devices(
    device_infos: &[MachineDeviceInfo],
    mapped_ecat_devices: &[EtherCATSubDevice],
) -> Vec<(MachineDeviceInfo, Rc<RefCell<dyn EthercatDevice>>)> {
    let mut result = Vec::new();

    for info in device_infos {
        for (meta, device) in mapped_ecat_devices {
            if meta.device_address == info.device_address {
                result.push((*info, device.clone()));
                break;
            }
        }
    }

    result.sort_by_key(|(info, _)| (info.machine_id, info.machine_serial));
    result
}
