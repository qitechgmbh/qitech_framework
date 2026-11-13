use std::cell::Cell;
use std::rc::Rc;

use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::request::WriteMachineDeviceInfoError;
use qitech_lib::ethercat_hal::ChannelError;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::BuildContext;
use crate::machine::hardware::MachineDeviceInfo;
use crate::resource::Journals;
use crate::resource::ResourceRegistry;
use crate::runtime::EtherCATController;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;
use crate::runtime::types::MachineRegistry;

pub fn find_machine(
    machines: &mut [MachineInstance],
    ident: MachineInstanceIdentification,
) -> Option<&mut MachineInstance> {
    machines.iter_mut().find(|instance| instance.ident == ident)
}

const VENDOR_WORD: u16 = 0x0028;
const MACHINE_WORD: u16 = 0x0029;
const SERIAL_WORD: u16 = 0x002a;
const ROLE_WORD: u16 = 0x002b;

pub fn read_identification(
    eth_control: &EtherCATController,
    device_address: u16,
) -> Result<MachineDeviceInfo, ChannelError> {
    Ok(MachineDeviceInfo {
        machine_vendor: eth_control.eeprom_read(device_address, VENDOR_WORD)?,
        machine_id: eth_control.eeprom_read(device_address, MACHINE_WORD)?,
        machine_serial: eth_control.eeprom_read(device_address, SERIAL_WORD)?,
        role: eth_control.eeprom_read(device_address, ROLE_WORD)?,
        device_address,
    })
}

fn write_identification(
    eth_control: &EtherCATController,
    device_address: u16,
    info: &MachineDeviceInfo,
) -> Result<(), ChannelError> {
    eth_control.eeprom_write(device_address, VENDOR_WORD, info.machine_vendor)?;
    eth_control.eeprom_write(device_address, MACHINE_WORD, info.machine_id)?;
    eth_control.eeprom_write(device_address, SERIAL_WORD, info.machine_serial)?;
    eth_control.eeprom_write(device_address, ROLE_WORD, info.role)?;
    Ok(())
}

pub fn write_machine_device_info(
    controller: &EtherCATController,
    machine_ident: MachineInstanceIdentification,
    role: u16,
    subdevice_index: usize,
) -> Result<(), WriteMachineDeviceInfoError> {
    let dev_addr = subdevice_index as u16;
    let m_serial = machine_ident.serial;
    let m_ident = machine_ident.machine;
    let ident = MachineDeviceInfo {
        role,
        machine_id: m_ident.machine_id,
        machine_vendor: m_ident.vendor_id,
        machine_serial: m_serial,
        device_address: dev_addr,
    };
    write_identification(controller, dev_addr, &ident).map_err(|err| {
        WriteMachineDeviceInfoError::WriteMachineDeviceInfoEeprom(err.to_string())
    })?;
    Ok(())
}

pub fn build_machines(
    export_count: Rc<Cell<u64>>,
    machine_registry: &MachineRegistry,
    hardware_registry: &HardwareRegistry,
    ecat_interface: Option<EtherCATThreadChannel>,
    journals: &mut Journals,
    resources: &mut ResourceRegistry,
    machine_instances: &mut Vec<MachineInstance>,
) -> Vec<(MachineInstanceIdentification, Result<(), BuildError>)> {
    let mut results: Vec<(MachineInstanceIdentification, Result<(), BuildError>)> = Vec::new();

    for (instance_id, hardware) in hardware_registry {
        // --- skip machines that are already instantiated ---
        if machine_instances.iter().any(|m| m.ident == *instance_id) {
            continue;
        }

        let ident = instance_id.machine;

        let Some(entry) = machine_registry.get(&ident) else {
            results.push((*instance_id, Err(BuildError::MachineTypeNotRegistered)));
            continue;
        };

        let mut ctx = BuildContext {
            ident: *instance_id,
            schema: &entry.schema,
            export_count: export_count.clone(),
            type_id: entry.type_id,
            type_name: entry.type_name,
            ethercat_interface: ecat_interface.clone(),
            hardware: hardware.clone(),
            journals,
            config: resources.config_properties.register(),
            state: resources.state_properties.register(),
            measurements: resources.measurements.register(),
            journals_temp: Journals::default(),
            config_registered: Default::default(),
            state_registered: Default::default(),
            measurements_registered: Default::default(),
            commands_registered: Default::default(),
            events_registered: Default::default(),
        };

        let machine = match (entry.build)(&mut ctx) {
            Ok(v) => v,
            Err(e) => {
                results.push((*instance_id, Err(e)));

                continue;
            }
        };

        // --- commit allocations ---
        ctx.config.commit();
        ctx.state.commit();
        ctx.measurements.commit();

        // --- import records from temp journal into export journal ---
        ctx.journals
            .config_property
            .import(ctx.journals_temp.config_property);

        ctx.journals
            .state_property
            .import(ctx.journals_temp.state_property);

        ctx.journals.command.import(ctx.journals_temp.command);
        ctx.journals.event.import(ctx.journals_temp.event);

        // --- extract metadata ---
        let configs = ctx.config_registered;
        let commands = ctx.commands_registered;

        // --- create instance ---
        machine_instances.push(MachineInstance {
            ident: *instance_id,
            machine,
            configs,
            commands,
            subscriptions: Default::default(),
        });

        // --- record outcome ---
        results.push((*instance_id, Ok(())));
    }

    results
}
