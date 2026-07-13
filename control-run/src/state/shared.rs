use std::collections::HashMap;

use qitech_lib::ethercat_hal::{Consumer, EtherCATControl, EtherCATThreadChannel, Producer, machine_ident_read::MachineDeviceInfo};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub struct SharedAppState {
    pub ethercat_meta_datas: RwLock<Vec<EtherCatDeviceMetaData>>,
    pub ethercat_thread_channel: Option<EtherCATThreadChannel>,
}

impl SharedAppState {
    pub fn fill_ethercat_metadata<C: Consumer, P: Producer>(
        &self,
        controller: &EtherCATControl<C, P>,
        infos: Vec<MachineDeviceInfo>,
    ) -> Result<(), anyhow::Error> {
        let mut guard = self.ethercat_meta_datas.try_write()?;
        let subdevices = controller.app_handle.try_get_subdevices_vec_sync()?;
        for dev in subdevices {
            let device_machine_identification = infos
                .iter()
                .find(|info| info.device_address == dev.device_address)
                .map(|info| DeviceMachineIdentification::from(*info));

            guard.push(
                EtherCatDeviceMetaData {
                    configured_address: dev.device_address,
                    name: dev.get_name()?,
                    vendor_id: dev.vendor,
                    product_id: dev.product_id,
                    revision: dev.revision,
                    device_identification: DeviceIdentification{
                            device_machine_identification,
                            device_hardware_identification:
                                machine_implementations::machine_identification::DeviceHardwareIdentification::Ethercat(DeviceHardwareIdentificationEthercat{ subdevice_index: dev.device_address as usize })
                    }
            });
        }
        drop(guard);
        Ok(())
    }

    /// Emit an EtherCAT interface discovery event synchronously (non-async).
    /// Uses `try_write()` so it can be called from a synchronous context during
    /// startup, before the Tokio runtime and Socket.io server are fully running.
    /// Events are queued and delivered when the Socket.io queue consumer starts.
    pub fn emit_ethercat_interface_discovery(
        &self,
        event: EthercatInterfaceDiscoveryEvent,
    ) -> Result<(), anyhow::Error> {
        let built = event.build();
        let mut guard = self.socketio_setup.namespaces.try_write()?;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatInterfaceDiscoveryEvent(built));
        drop(guard);
        Ok(())
    }

    pub async fn send_ethercat_setup_init(&self) {
        let event = Event::new(
            "EthercatDevicesEvent",
            EthercatDevicesEvent::Initializing(true),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn send_ethercat_setup_done(&self) {
        let event = Event::new(
            "EthercatDevicesEvent",
            EthercatDevicesEvent::Done(EthercatSetupDone {
                devices: self.ethercat_meta_datas.read().await.clone(),
            }),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn send_machines_event(&self) -> Result<(), anyhow::Error> {
        let event = MachinesEventBuilder().build(self.get_machines_meta().await);
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
        drop(guard);
        Ok(())
    }

    pub async fn send_ethercat_state(&self, ecat_state: EcatState) {
        let event = Event::new(
            "EthercatStateEvent",
            EthercatDevicesEvent::State(ecat_state.into()),
        );
        let mut guard = self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::EthercatDevicesEvent(event));
        drop(guard);
    }

    pub async fn get_machines_meta(&self) -> Vec<MachineObj> {
        self.machines.read().await.clone()
    }

    pub async fn message_machine(
        &self,
        machine_identification_unique: &QiTechMachineIdentificationUnique,
        message: MachineMessage,
    ) -> Result<(), anyhow::Error> {
        let guard = self.machines_with_channel.read().await;
        let sender = guard.get(machine_identification_unique);
        if let Some(sender) = sender {
            sender.send(message).await?;
        }
        drop(guard);
        // why does a macro for return Err() exist bro ...
        bail!("Unknown machine!")
    }

    pub async fn add_machine(
        &self,
        ident: QiTechMachineIdentificationUnique,
        err: Option<String>,
        sender: Sender<MachineMessage>,
    ) {
        let mut guard = self.machines.write().await;
        let machine_obj = MachineObj {
            machine_identification_unique: ident,
            error: err,
        };
        guard.push(machine_obj);
        drop(guard);

        let mut guard = self.machines_with_channel.write().await;
        guard.insert(ident, sender);
        drop(guard);
    }

    pub fn new() -> Self {
        let (socket_queue_tx, socket_queue_rx) = tokio::sync::mpsc::channel(64);
        Self {
            machines: RwLock::new(vec![]),
            machines_with_channel: RwLock::new(HashMap::new()),
            socketio_setup: SocketioSetup {
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new(socket_queue_tx.clone())),
                socket_queue_tx,
                socket_queue_rx: RwLock::new(socket_queue_rx),
            },
            ethercat_meta_datas: RwLock::new(vec![]),
            ethercat_thread_channel: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EtherCatDeviceMetaData {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}