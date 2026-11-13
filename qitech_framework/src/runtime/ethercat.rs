use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use super::error::EtherCATInitializeError;
use super::error::EtherCATInitializeResult;
use super::error::RuntimeInitializeResult;
use crate::machine::Hardware;
use crate::machine::hardware::EtherCATDeviceIdentified;
use crate::machine::hardware::MachineDeviceInfo;
use crate::runtime::EtherCATConfig;
use crate::runtime::EtherCATController;
use crate::runtime::EtherCATSubDevice;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::utils::read_identification;
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
use qitech_lib::ethercat_hal::ChannelError;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::convert_module_to_meta_ethercat_device;
use qitech_lib::ethercat_hal::interface_discovery::LinkType;
use qitech_lib::ethercat_hal::interface_discovery::list_ethernet_interfaces;
use qitech_lib::ethercat_hal::interface_discovery::test_interface;
use qitech_lib::ethercat_hal_devices::EthercatDevice;
use qitech_lib::ethercat_hal_devices::beckhoff_modules::{
    ek1100::EK1100, el1002::EL1002, el1008::EL1008, el1124::EL1124, el1259::EL1259, el2002::EL2002,
    el2004::EL2004, el2008::EL2008, el2521::EL2521, el2522::EL2522, el3001::EL3001, el3021::EL3021,
    el3024::EL3024, el3062_0030::EL3062_0030, el3204::EL3204, el4002::EL4002, el4008::EL4008,
    el4732::EL4732, el5152::EL5152, el6021::EL6021, el7031::EL7031, el7031_0030::EL7031_0030,
    el7037::EL7037, el7041_0052::EL7041_0052, ep2339_0021::EP2339_0021,
};
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_354::Wago750_354;
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_402::Wago750_402;
use qitech_lib::ethercat_hal_devices::wago_modules::wago_750_531::Wago750_531;
use qitech_lib::ethercat_hal_devices::{IdentifiedDevice, NewEthercatDevice};

/// No driver matches the subdevice's vendor and product id
#[derive(Debug)]
pub struct NoDriverError {
    pub vendor: u32,
    pub product_id: u32,
    pub revision: u32,
}

impl std::fmt::Display for NoDriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "No Driver: vendor_id: 0x{:x}, product_id: 0x{:x}, revision: 0x{:x}",
            self.vendor, self.product_id, self.revision
        )
    }
}

impl std::error::Error for NoDriverError {}

/// Returns a new driver for the first type whose `get_identity()` return value matches the subdevice
macro_rules! driver_for {
    ($dev:expr; $($driver:ty),* $(,)?) => {
        $(
            if <$driver as IdentifiedDevice>::identity().matches($dev) {
                return Ok(Rc::new(RefCell::new(<$driver as NewEthercatDevice>::new())));
            }
        )*
    };
}

pub fn device_from_subdevice_identity_rc(
    dev: &MetaSubdevice,
) -> Result<Rc<RefCell<dyn EthercatDevice>>, NoDriverError> {
    driver_for!(dev;
        Wago750_354,
        Wago750_531,
        Wago750_402,
        EK1100,
        EL1002,
        EL1008,
        EL1124,
        EL1259,
        EL2002,
        EL2004,
        EL2008,
        EL2521,
        EL2522,
        EL3001,
        EL3021,
        EL3024,
        EL3062_0030,
        EL3204,
        EL4002,
        EL4008,
        EL4732,
        EL5152,
        EL6021,
        EL7031,
        EL7031_0030,
        EL7037,
        EL7041_0052,
        EP2339_0021,
    );

    Err(NoDriverError {
        vendor: dev.vendor,
        product_id: dev.product_id,
        revision: dev.revision,
    })
}

#[tracing::instrument(skip_all)]
pub fn init<T: RuntimeTransport>(
    config: &EtherCATConfig,
    session: &mut SessionInitializing<T>,
    hardware_registry: &mut HardwareRegistry,
) -> RuntimeInitializeResult<(Option<EtherCATController>, Vec<EtherCATSubDevice>)> {
    session.send_event(RuntimeInitEvent::EtherCATDiscoveryStarted)?;

    let interface = find_interface(config.interface_scan_interval);

    session.send_event(RuntimeInitEvent::EtherCATDiscoveryCompleted {
        interface: interface.clone(),
    })?;

    let controller = ethercat_hal::init_ethercat(&interface, Some(config.master_config.clone()));

    let state = match controller.get_state() {
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

    let state = match controller.get_state() {
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
        .request_state_change(ethercat_hal::EtherCATState::PreOp)
        .map_err(|arg0: ChannelError| {
            EtherCATInitializeError::FailedToRequestStateChange(arg0.into())
        })?;

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

        let preop_ready = controller.get_state() == ethercat_hal::EtherCATState::PreOp
            && controller.get_subdevice_count() > 0;

        if preop_ready {
            stable_ticks += 1
        } else {
            stable_ticks = 0
        }
    }

    tracing::info!(
        count = controller.get_subdevice_count(),
        "initialized subdevices"
    );

    let meta_subdevices =
        controller
            .try_get_subdevices_vec_sync()
            .map_err(|arg0: ChannelError| {
                EtherCATInitializeError::FailedToGetSubDevices(arg0.into())
            })?;

    // Find the Wago 750-354 couplers offsets
    let couplers: Vec<_> = meta_subdevices
        .iter()
        .filter(|s| Wago750_354::identity().matches(s))
        .collect();

    if !couplers.is_empty() {
        tracing::debug!("WAGO Coupler found! Initializing WAGO modules");
    }

    let modules_as_devices: Vec<_> = couplers
        .iter()
        .flat_map(|coupler_subdev| {
            // Devices connected to 750-354 are not actual ethercat terminals and are commonly called "modules"
            let modules = controller.get_channel().get_modules(
                coupler_subdev.device_address,
                coupler_subdev.vendor,
                None,
            );

            if let Err(err) = &modules {
                tracing::error!(
                    "Failed to read WAGO module data from coupler at {:?}: {}",
                    coupler_subdev.device_address,
                    err
                );
            }

            modules.into_iter().flat_map(|modules| {
                modules.into_iter().map(|module| {
                    let coupler_start_tx = coupler_subdev.start_tx;
                    let coupler_start_rx = coupler_subdev.start_rx;
                    convert_module_to_meta_ethercat_device(
                        &module,
                        coupler_start_tx,
                        coupler_start_rx,
                    )
                })
            })
        })
        .collect();

    let meta_subdevices = &[meta_subdevices, modules_as_devices].concat();

    let mut subdevices = Vec::new();

    for meta in meta_subdevices {
        let dev = match device_from_subdevice_identity_rc(meta) {
            Ok(d) => d,
            Err(_) => {
                tracing::warn!(name = ?meta.get_name(), "no EtherCAT device implementation");
                continue;
            }
        };

        subdevices.push((*meta, dev));
        if meta.vendor == BECKHOFF_VENDOR_ID {
            controller
                .set_mut_beckhoff_eeprom_lock_active(meta.device_address)
                .map_err(|arg0: ChannelError| {
                    EtherCATInitializeError::FailedToSetBeckhoffEepromLockActive(arg0.into())
                })?;
        }
    }

    Ok(subdevices)
}

#[tracing::instrument(skip_all)]
pub fn finalize(
    controller: &EtherCATController,
    sub_devices: &mut Vec<EtherCATSubDevice>,
) -> RuntimeInitializeResult<()> {
    controller
        .request_state_change(ethercat_hal::EtherCATState::Op)
        .map_err(|arg0: ChannelError| {
            EtherCATInitializeError::FailedToRequestStateChange(arg0.into())
        })?;

    wait_for_op_state(controller)?;

    // update offsets in sub devices
    let op_subdevices = controller
        .try_get_subdevices_vec_sync()
        .map_err(|err: ChannelError| EtherCATInitializeError::FailedToGetSubDevices(err.into()))?;

    for (meta, _) in sub_devices {
        let Some(src) = find_subdevice(&op_subdevices, meta.device_address) else {
            return Err(RuntimeInitializeError::AssertionFailed(
                "EtherCAT device suddenly missing in finalize",
            ));
        };

        update_sub_device_offsets(meta, src);
    }

    Ok(())
}

fn find_subdevice(sub_devices: &[MetaSubdevice], device_address: u16) -> Option<&MetaSubdevice> {
    sub_devices.iter().find(|&device|
            // We don't want to update modules, only their coupler, if any
            device.module_index.is_none() && device.device_address == device_address)
}

fn update_sub_device_offsets(dest: &mut MetaSubdevice, src: &MetaSubdevice) {
    dest.start_tx = src.start_tx;
    dest.end_tx = src.end_tx;

    dest.start_rx = src.start_rx;
    dest.end_rx = src.end_rx;
}

// --- utils ---
pub fn is_controller_finished(controller: &EtherCATController) -> bool {
    match &controller.join_handle {
        Some(handle) => handle.is_finished(),
        None => false,
    }
}

fn wait_for_op_state(controller: &EtherCATController) -> EtherCATInitializeResult<()> {
    while !controller.check_all_op() {
        if is_controller_finished(controller) {
            return Err(EtherCATInitializeError::FailedToReachOpState);
        }
        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

// --- deconstructing ---
fn read_and_register_identifications(
    controller: &EtherCATController,
    subdevices: &[EtherCATSubDevice],
    hardware_registry: &mut HardwareRegistry,
) -> Vec<MachineDeviceInfo> {
    let mut mapped_devices = Vec::new();

    for subdevice in subdevices.iter() {
        // FIXME: RETURN AN ERROR HERE
        let mut ident = read_identification(controller, subdevice.0.device_address).unwrap();

        // For modules, the bus coupler has role zero, but anything else,
        // should have a different role.
        // We cannot write the role to modules directly.
        if let Some(module_index) = subdevice.0.module_index {
            ident.role = module_index + 1;
        }
        mapped_devices.push((ident, subdevice.clone()));
    }

    append_ethercat(hardware_registry, mapped_devices)
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
    device_infos: Vec<(MachineDeviceInfo, EtherCATSubDevice)>,
) -> Vec<MachineDeviceInfo> {
    let mut infos = Vec::new();

    for (info, (_, device)) in device_infos.into_iter() {
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

        infos.push(info);
    }

    infos
}
