use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::session::RuntimeTransport;

use crate::Runtime;
use crate::machine::Machine;
use crate::runtime::utils;
use crate::runtime::utils::find_machine;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn process_requests(&mut self) {
        for _ in 0..self.config.requests_per_cycle_max {
            let Some(req) = self.session.recv_request().unwrap() else {
                break;
            };

            let kind = req.request_id;
            let response = self.process_request(req);
            self.report.responses.push((kind, response));
        }
    }

    fn process_request(&mut self, request: RuntimeRequest) -> Result<(), String> {
        // let request_id = request.request_id;

        match request.kind {
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
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    return Err("No Such Machine".to_string());
                };

                let machine_ref: &mut dyn Machine = &mut *machine;

                self.resources
                    .config_properties
                    .write_value(
                        target, 
                        &path, 
                        machine_ref,
                        value,
                    )
                    .map_err(|e| format!("{e}")).unwrap();

                Ok(())
            }

            RuntimeRequestKind::InvokeMachineCommand { target, resource } => {
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    return Err("No Such Machine".to_string());
                };

                let machine_ref: &mut dyn Machine = &mut *machine;

                let result = self
                    .resources
                    .commands
                    .invoke(target, machine_ref, &resource)
                    .unwrap();

                /*
                self.report
                    .machines
                    .command_traces
                    .push(MachineCommandTrace {
                        request_id,
                        ident: target,
                        resource,
                        timestamp: Utc::now(),
                        result,
                    });
                    */

                Ok(())
            }

            RuntimeRequestKind::MachineSubscribe {
                provider,
                subscriber,
            } => {
                /*
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
                */

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
