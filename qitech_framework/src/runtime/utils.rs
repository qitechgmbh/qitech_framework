use std::cell::Cell;
use std::fs;
use std::rc::Rc;

use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::request::ReadMachineDeviceInfoError;
use qitech_framework_core::request::WriteMachineDeviceInfoError;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::BuildContext;
use crate::resource::Journals;
use crate::resource::ResourceRegistry;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineRegistry;
use crate::runtime::EtherCATController;
use crate::runtime::types::MachineInstance;

pub fn find_machine(
    machines: &mut [MachineInstance],
    ident: MachineInstanceIdentification,
) -> Option<&mut MachineInstance> {
    machines.iter_mut().find(|instance| instance.ident == ident)
}

pub fn write_machine_device_info(
    controller: &EtherCATController,
    machine_ident: MachineInstanceIdentification,
    role: u16,
    subdevice_index: usize,
) -> Result<(), WriteMachineDeviceInfoError> {
    let mut idents = read_machine_device_info()?;

    let dev_addr = subdevice_index as u16;
    let ident = idents.iter_mut().find(|i| i.device_address == dev_addr);

    let m_serial = machine_ident.serial;
    let m_ident = machine_ident.machine;

    if let Some(ident) = ident {
        ident.role = role;
        ident.machine_vendor = m_ident.vendor_id;
        ident.machine_id = m_ident.machine_id;
        ident.machine_serial = m_serial;
    } else {
        idents.push(MachineDeviceInfo {
            role,
            machine_id: m_ident.machine_id,
            machine_vendor: m_ident.vendor_id,
            machine_serial: m_serial,
            device_address: dev_addr,
        });
    }

    controller
        .channel
        .write_machine_device_info_eeprom(idents)
        .map_err(|e| WriteMachineDeviceInfoError::WriteMachineDeviceInfoEeprom(e.to_string()))?;

    Ok(())
}

pub fn read_machine_device_info() -> Result<Vec<MachineDeviceInfo>, ReadMachineDeviceInfoError> {
    let path = get_machine_device_info_path();

    let exists = fs::exists(&path).map_err(|_| ReadMachineDeviceInfoError::CheckExists)?;
    if !exists {
        return Ok(vec![]);
    }

    let json = fs::read_to_string(&path).map_err(|_| ReadMachineDeviceInfoError::ReadFile)?;

    let value = serde_json::to_value(&json).map_err(|_| ReadMachineDeviceInfoError::ParseJson)?;

    let infos = value
        .as_array()
        .ok_or(ReadMachineDeviceInfoError::RootNotArray)?
        .iter()
        .map(
            |value| -> Result<MachineDeviceInfo, ReadMachineDeviceInfoError> {
                Ok(MachineDeviceInfo {
                    role: value["role"].as_u64().unwrap_or(0) as u16,
                    machine_id: value["machine_id"].as_u64().unwrap_or(0) as u16,
                    machine_vendor: value["machine_vendor"].as_u64().unwrap_or(0) as u16,
                    machine_serial: value["machine_serial"].as_u64().unwrap_or(0) as u16,
                    device_address: value["device_address"]
                        .as_u64()
                        .ok_or(ReadMachineDeviceInfoError::MissingDeviceAddress)?
                        as u16,
                })
            },
        )
        .collect::<Result<Vec<_>, _>>()?;

    Ok(infos)
}

fn get_machine_device_info_path() -> String {
    let dir = std::env::var("STATE_DIRECTORY")
        .or(std::env::var("XDG_DATA_HOME"))
        .or(std::env::var("HOME"))
        .unwrap_or(".".to_string());

    dir + "/qitech.json"
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
