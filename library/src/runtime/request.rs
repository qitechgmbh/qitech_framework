use qitech_framework_common::RuntimeRequestKind;

use crate::Runtime;
use crate::machine::Machine;
use crate::runtime::Bridge;
use crate::runtime::utils;
use crate::runtime::utils::find_machine;

impl<B: Bridge> Runtime<B> {
    pub fn process_requests(&mut self) {

        /*
        for req in self.bridge.get_requests(self.config.requests_per_cycle_max) {
            let response = self.process_request(req.kind);
            self.report.responses.push((req.transaction_id, response));
        }
        */
    }

    fn process_request(&mut self, kind: RuntimeRequestKind) -> Result<(), String> {
        Ok(())
        /*
        match kind {
            RuntimeRequestKind::WriteMachineDeviceInfo {
                machine_ident,
                role,
                subdevice_index,
            } => {
                let Some(controller) = &self.ecat_controller else {
                    return Err("No EtherCAT controller available".into());
                };

                // TODO: submit error !
                _ = utils::write_machine_device_info(
                    controller,
                    machine_ident,
                    role,
                    subdevice_index,
                );

                return Ok(());
            }

            RuntimeRequestKind::SetMachineConfiguration(..) => {
                // self.resources.config_properties;
                return Ok(());
            }

            RuntimeRequestKind::InvokeMachineCommand {
                target,
                resource_path,
                arguments,
            } => {
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    return Err("No Such Machine".to_string());
                };

                let machine_ref: &mut dyn Machine = &mut *machine;

                let res = self.resources.commands.invoke(
                    target,
                    machine_ref,
                    &resource_path,
                    &arguments,
                );

                let result = res.map_err(|e| format!("{e}"));
                self.report.responses.push((req.transaction_id, result));
            }

            RuntimeRequestKind::MachineSubscribe { provider, consumer } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    return Err("No Such Machine".to_string());
                }

                // --- find consumer ---
                let Some(machine) = find_machine(&mut self.machines, consumer) else {
                    return Err("No Such Machine".to_string());
                };

                let entry = self.subscriptions.entry(consumer);

                if entry.

                let ctx = SubscribeContext::new(provider, &mut self.resources);

                let result = machine.subscribe(&ctx).map_err(|e| format!("{e}"));

                if result.is_ok() {

                }

                self.report
                    .responses
                    .push((req.transaction_id, result));
            }

            RuntimeRequestKind::MachineUnsubscribe { provider, consumer } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    return Err("No Such Machine")
                }

                // --- find consumer ---
                let Some(machine) = find_machine(&mut self.machines, consumer) else {
                    self.report
                        .responses
                        .push((req.transaction_id, Err(format!("No Such Machine {provider}"))));

                    return;
                };

                let Some(entry) = self.subscriptions.get_mut(&consumer) else {


                    return;
                };

                return Ok(());
            }
        }
        */
    }
}
