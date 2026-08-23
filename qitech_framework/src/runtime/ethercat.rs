use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::ident::DeviceHardwareIdentification;
use qitech_framework_core::ident::DeviceHardwareIdentificationEthercat;
use qitech_framework_core::ident::DeviceIdentification;
use qitech_framework_core::ident::DeviceMachineIdentification;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
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
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineIdentificationPreset;

struct EtherCATSubDevice {
    pub meta: MetaSubdevice,
    pub handle: Rc<RefCell<dyn EthercatDevice + 'static>>,
}

/// EtherCAT devices identified by some mechanism are accumulated into this map.
///
/// We use the device address as key. As such a device may ever only get mapped once.
type IdentifiedDevicesByAddress = HashMap<u16, EtherCATDeviceIdentified>;

#[tracing::instrument(skip_all)]
pub fn init<T: RuntimeTransport>(
    config: EtherCATConfig,
    session: &mut SessionInitializing<T>,
    hardware_registry: &mut HardwareRegistry,
) -> RuntimeInitializeResult<(EtherCATController, Vec<EtherCATDeviceIdentified>)> {
    session.send_event(RuntimeInitEvent::EtherCATDiscoveryStarted)?;

    let interface = find_interface(config.interface_scan_interval);

    session.send_event(RuntimeInitEvent::EtherCATDiscoveryCompleted {
        interface: interface.clone(),
    })?;

    let controller = ethercat_hal::init_ethercat(&interface, Some(config.master_config));
    let state = get_ethercat_state(&controller);

    session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;
    session.send_event(RuntimeInitEvent::EtherCATInitializationStarted)?;
    let subdevices = setup(&controller)?;

    let state = get_ethercat_state(&controller);
    session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;

    let mut devices_by_address = IdentifiedDevicesByAddress::new();

    identify_devices_by_presets(&mut devices_by_address, &subdevices, &config.preset_idents);

    if config.assign_devices_by_eeprom_read {
        let eeprom_data = read_eeprom_identifications(&controller);
        identify_devices_by_eeprom(&mut devices_by_address, &subdevices, &eeprom_data);
    }

    send_ecat_metadata(session, &devices_by_address)?;

    register_devices(hardware_registry, &devices_by_address);

    let subdevices = devices_by_address.into_values().collect();
    Ok((controller, subdevices))
}

fn get_ethercat_state(controller: &EtherCATController) -> EtherCATStatus {
    match controller.app_handle.get_state() {
        ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
        ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
        ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
        ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
        ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
        ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
    }
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
fn setup(controller: &EtherCATController) -> RuntimeInitializeResult<Vec<EtherCATSubDevice>> {
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
        let handle = match device_from_subdevice_identity_rc(&meta) {
            Ok(d) => d,
            Err(_) => {
                tracing::warn!(name = ?meta.get_name(), "no EtherCAT device implementation");
                continue;
            }
        };

        subdevices.push(EtherCATSubDevice { meta, handle: handle.clone() });

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
    subdevices: &mut Vec<EtherCATDeviceIdentified>,
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

    for subdevice in subdevices {
        let meta = &mut subdevice.meta;
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

fn find_subdevice(subdevices: &[MetaSubdevice], device_address: u16) -> Option<&MetaSubdevice> {
    subdevices
        .iter()
        .find(|&device| device.device_address == device_address)
}

fn update_sub_device_offsets(dest: &mut MetaSubdevice, src: &MetaSubdevice) {
    dest.start_tx = src.start_tx;
    dest.end_tx = src.end_tx;

    dest.start_rx = src.start_rx;
    dest.end_rx = src.end_rx;
}

// --- deconstructing ---
fn read_eeprom_identifications(controller: &EtherCATController) -> Vec<MachineDeviceInfo> {
    match controller.channel.read_device_identifications() {
        Ok(eeprom_idents) => return eeprom_idents,
        Err(err) => {
            tracing::error!(?err, "could not read device identifications from eeprom");
        }
    }

    Vec::new()
}

fn register_devices(hardware_registry: &mut HardwareRegistry, devices_by_address: &IdentifiedDevicesByAddress) {
    for (_, device) in devices_by_address.into_iter() {
        hardware_registry
            .entry(device.ident)
            .or_default()
            .push(Hardware::Ethercat(device.clone()));

    }
}

fn send_ecat_metadata<T: RuntimeTransport>(
    session: &mut SessionInitializing<T>,
    devices_by_address: &IdentifiedDevicesByAddress,
) -> RuntimeInitializeResult<()> {

    let ecat_metadata = devices_by_address
        .iter()
        .map(|(addr, device)| EtherCATDeviceMetadata {
            configured_address: *addr,
            name: device.meta.get_name().expect("please be a utf-8"),
            vendor_id: device.meta.vendor,
            product_id: device.meta.product_id,
            revision: device.meta.revision,
            device_identification: DeviceIdentification {
                device_machine_identification: Some(DeviceMachineIdentification {
                    machine_ident: device.ident,
                    role: device.role.unwrap_or_default(),
                }),
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat {
                        subdevice_index: *addr as usize,
                    },
                ),
            },
        })
        .collect();

    session.send_event(RuntimeInitEvent::EtherCATDeviceInitializationCompleted {
        devices: ecat_metadata,
    })?;

    Ok(())
}

fn identify_devices_by_presets(
    devices_by_address: &mut IdentifiedDevicesByAddress,
    subdevices: &[EtherCATSubDevice],
    presets: &[MachineIdentificationPreset],
) {
    for preset in presets {
        let Some(subdevice) = subdevices.iter().find(|sub| preset.matches(&sub.meta)) else {
            tracing::debug!(
                "Subdevice with vendor={}, product={} for preset not found, moving on...",
                preset.vendor_id,
                preset.product_id
            );
            continue;
        };

        devices_by_address.insert(
            subdevice.meta.device_address,
            EtherCATDeviceIdentified {
                meta: subdevice.meta,
                handle: subdevice.handle.clone(),
                ident: preset.ident,
                role: None,
            },
        );
    }
}

fn identify_devices_by_eeprom(
    devices_by_address: &mut IdentifiedDevicesByAddress,
    subdevices: &[EtherCATSubDevice],
    eeprom_data: &[MachineDeviceInfo],
) {
    for info in eeprom_data {
        let addr = info.device_address;
        if devices_by_address.contains_key(&addr) {
            tracing::debug!(
                "Device {:04x} already identified! Ignoring identity from eeprom.",
                addr
            );
            continue;
        }

        let subdevice = subdevices
            .iter()
            .find(|sub| sub.meta.device_address == addr)
            .expect("A subdevice that reported its identiy via the eeprom must exist!");

        let ident = MachineIdentificationUnique {
            identification: MachineIdentification {
                vendor_id: info.machine_vendor,
                machine_id: info.machine_id,
            },
            serial: info.machine_serial,
        };

        devices_by_address.insert(
            addr,
            EtherCATDeviceIdentified {
                meta: subdevice.meta,
                handle: subdevice.handle.clone(),
                ident,
                role: Some(info.role),
            },
        );
    }
}
