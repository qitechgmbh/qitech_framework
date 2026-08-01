use qitech_framework_common::RuntimeRequestKind;
use qitech_framework_common::session::RuntimeTransport;

use crate::Runtime;
use crate::machine::Machine;
use crate::machine::SubscribeContext;
use crate::runtime::utils;
use crate::runtime::utils::find_machine;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn process_requests(&mut self) {
        for _ in 0..self.config.requests_per_cycle_max {
            let Some(req) = self.session.recv_request().unwrap() else {
                break;
            };

            let response = self.process_request(req.kind);
            self.report.responses.push((req.transaction_id, response));
        }
    }

    fn process_request(&mut self, kind: RuntimeRequestKind) -> Result<(), String> {
        match kind {
            RuntimeRequestKind::WriteMachineDeviceInfo {
                machine_ident,
                role,
                subdevice_index,
            } => {
                let Some(controller) = &self.ecat_controller else {
                    return Err("No EtherCAT controller available".into());
                };

                // TODO: submit error
                _ = utils::write_machine_device_info(
                    controller,
                    machine_ident,
                    role,
                    subdevice_index,
                );

                Ok(())
            }

            RuntimeRequestKind::SetMachineConfiguration {
                target,
                resource: path,
                value,
            } => {
                self.resources
                    .config_properties
                    .api_write(
                        0, // TODO: provide the real one
                        target, &path, value,
                    )
                    .map_err(|e| format!("{e}"))
            }

            RuntimeRequestKind::InvokeMachineCommand {
                target,
                resource,
                arguments,
            } => {
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    return Err("No Such Machine".to_string());
                };

                let machine_ref: &mut dyn Machine = &mut *machine;

                let res =
                    self.resources
                        .commands
                        .invoke(target, machine_ref, &resource, &arguments);

                res.map_err(|e| format!("{e}"))
            }

            RuntimeRequestKind::MachineSubscribe {
                provider,
                subscriber,
            } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    return Err("No Such Machine".to_string());
                }

                // --- find consumer ---
                let Some(machine) = find_machine(&mut self.machines, subscriber) else {
                    return Err("No Such Machine".to_string());
                };

                // --- prevent duplicate subscription ---
                let subscribers = self.subscriptions.entry(provider).or_default();

                if subscribers.contains(&subscriber) {
                    return Err("Already subscribed".to_string());
                }

                // --- let machine allocate resources ---
                let ctx = SubscribeContext::new(provider, subscriber, &mut self.resources);

                if let Err(e) = machine.subscribe(ctx).map_err(|e| e.to_string()) {
                    // failed, clean up any created handles
                    self.resources.remove_subscription(provider, subscriber);
                    return Err(e);
                }

                // --- register subscription ---
                subscribers.push(subscriber);

                Ok(())
            }

            RuntimeRequestKind::MachineUnsubscribe {
                provider,
                subscriber,
            } => {
                let Some(entry) = self.subscriptions.get_mut(&provider) else {
                    return Err("No such provider".to_string());
                };

                entry.retain(|v| *v != subscriber);
                self.resources.remove_subscription(provider, subscriber);

                Ok(())
            }
        }
    }
}
