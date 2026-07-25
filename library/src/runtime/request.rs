use qitech_framework_common::OperationResult;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::RuntimeRequestKind;

use crate::Runtime;
use crate::machine::Machine;
use crate::machine::SubscribeContext;
use crate::runtime::utils::find_machine;

impl Runtime {
    fn handle_request(&mut self, request: RuntimeRequest) {
        match request.kind {
            RuntimeRequestKind::WriteMachineDeviceInfo { .. } => {}

            RuntimeRequestKind::SetMachineConfiguration(..) => {
                // self.resources.config_properties;
            }

            RuntimeRequestKind::InvokeMachineCommand {
                target,
                resource_path,
                arguments,
            } => {
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));

                    return;
                };

                let machine_ref: &mut dyn Machine = &mut *machine;

                let res =
                    self.resources
                        .commands
                        .invoke(target, machine_ref, &resource_path, &arguments);

                // TODO: create logs with transaction id as tag
                let response = match res {
                    Ok(_) => OperationResult::Success,
                    Err(_) => OperationResult::Failure,
                };

                self.report
                    .responses
                    .push((request.transaction_id, response));
            }

            RuntimeRequestKind::MachineSubscribe { provider, consumer } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));
                }

                // --- find consumer ---
                let Some(machine) = find_machine(&mut self.machines, consumer) else {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));

                    return;
                };

                let ctx = SubscribeContext::new(provider, &mut self.resources);

                // TODO: do something with this I guess
                match machine.subscribe(&ctx) {
                    Ok(_) => {
                        // self.subscriptions.insert(provider, consumer);
                    },
                    Err(_) => todo!(),
                }
            }

            RuntimeRequestKind::MachineUnsubscribe { provider, consumer } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));
                }

                // --- find consumer ---
                let Some(machine) = find_machine(&mut self.machines, consumer) else {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));

                    return;
                };
            },
        }
    }
}
