use qitech_framework_common::OperationResult;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::RuntimeRequestKind;

use crate::Runtime;
use crate::machine::Machine;

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
                let Some((_, machine)) =
                    self.machines.iter_mut().find(|(ident, _)| *ident == target)
                else {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));
                    return;
                };

                let machine_ref: &mut dyn Machine = &mut **machine;
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
            RuntimeRequestKind::MachineSubscribe {
                provider: source,
                consumer: subscriber,
            } => {
                let Some((_, machine)) = self
                    .machines
                    .iter_mut()
                    .find(|(ident, _)| *ident == subscriber)
                else {
                    self.report
                        .responses
                        .push((request.transaction_id, OperationResult::Failure));
                    return;
                };

                let ctx = todo!();
                if let Err(e) = machine.subscribe(ctx) {}
            }

            RuntimeRequestKind::MachineUnsubscribe { .. } => todo!(),
        }
    }
}
