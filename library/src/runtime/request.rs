use chrono::Utc;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::CommandExecuteError;
use qitech_framework_core::report::CommandRecord;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::report::ResourceAccessError;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::RuntimeEvent;
use qitech_framework_core::request::MachineExecuteCommandError;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::request::RuntimeResponse;
use qitech_framework_core::request::MachineSubscribeError;
use qitech_framework_core::request::MachineUnsubscribeError;
use qitech_framework_core::request::MachineSetConfigProperty;
use qitech_framework_core::request::WriteMachineDeviceInfoError;
use qitech_framework_core::session::RuntimeTransport;

use crate::Runtime;
use crate::machine::LifetimeTokenOwner;
use crate::machine::SubscribeContext;
use crate::runtime::utils;
use crate::runtime::utils::find_machine;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn process_requests(&mut self) {
        for _ in 0..self.config.requests_per_cycle_max {
            let Some(req) = self.session.recv_request().unwrap() else {
                break;
            };

            let request_id = req.request_id;
            let result = self.process_request(req);

            self.report
                .responses
                .push(RuntimeResponse { request_id, result });
        }
    }

    fn process_request(&mut self, request: RuntimeRequest) -> Result<(), RuntimeRequestError> {
        let request_id = request.request_id;

        match request.kind {
            RuntimeRequestKind::WriteMachineDeviceInfo {
                machine_ident,
                role,
                subdevice_index,
            } => {
                let Some(controller) = &self.ecat_controller else {
                    return Err(WriteMachineDeviceInfoError::NoEtherCATController)?;
                };

                Ok(utils::write_machine_device_info(
                    controller,
                    machine_ident,
                    role,
                    subdevice_index,
                )?)
            }

            RuntimeRequestKind::SetConfigProperty {
                target,
                path,
                value,
            } => {
                // --- find the machine ---
                let Some(instance) = find_machine(&mut self.machines, target) else {
                    return Err(MachineSetConfigProperty::ResourceAccess(
                        ResourceAccessError::MachineNotFound
                    ))?;
                };

                // --- find the handle ---
                let Some(handle) = instance.configs.get_mut(path.as_str()) else {
                    return Err(MachineSetConfigProperty::ResourceAccess(
                        ResourceAccessError::ResourceNotFound { 
                            kind: ResourceKind::ConfigProperty, 
                            path 
                        }
                    ))?;
                };

                // --- execute the write ---
                let result = (handle.write)(value.clone());

                // --- record the outcome ---
                let outcome = match result.clone() {
                    Ok(changed) => ConfigPropertyWriteOutcome::Accepted { changed },
                    Err(e) => ConfigPropertyWriteOutcome::Rejected(e),
                };

                let record = ConfigPropertyRecord {
                    timestamp: Utc::now(),
                    machine: target,
                    path: path.to_string(),
                    event: ConfigPropertyEvent::Written {
                        value,
                        origin: OperationOrigin::Request { request_id },
                        outcome,
                    },
                };

                self.journals.record_config(record);

                // --- process callback ---
                let callback = handle.on_changed.as_ref();

                if let Some(callback) = callback
                    && let Err(e) = callback(instance.machine.as_mut()) {
                        // TODO: remove machine
                        _ = e;
                    };

                // --- yield the result ---
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(MachineSetConfigProperty::WriteError(e))?,
                }
            }

            RuntimeRequestKind::ExecuteCommand {
                target,
                path,
            } => {
                // --- find the machine ---
                let Some(instance) = find_machine(&mut self.machines, target) else {
                    return Err(MachineExecuteCommandError::ResourceAccess(
                        ResourceAccessError::MachineNotFound
                    ))?;
                };

                // --- find the handle ---
                let Some(handle) = instance.commands.get_mut(path.as_str()) else {
                    return Err(MachineExecuteCommandError::ResourceAccess(
                        ResourceAccessError::ResourceNotFound { 
                            kind: ResourceKind::Command, 
                            path 
                        }
                    ))?;
                };

                // --- ensure we can execute the command ---
                if let Some(can_execute) = &handle.can_execute_fn {
                    let capability = (can_execute)(instance.machine.as_ref());

                    if let OperationCapability::Forbidden { reason } = &capability {
                        let reason = reason.clone();
                        let e = CommandExecuteError::Disabled { reason };

                        self.journals.commands.new_handle().append(CommandRecord {
                            timestamp: Utc::now(),
                            machine: target,
                            path,
                            event: CommandEvent::Executed(Err(e.clone())),
                        });
                        
                        return Err(RuntimeRequestError::MachineExecuteCommand(
                            MachineExecuteCommandError::ExecuteError(e),
                        ));
                    }
                }

                // --- execute it ---
                let result = (handle.execute_fn)(instance.machine.as_mut())
                    .map_err(CommandExecuteError::ExecutionError);

                self.journals.commands.new_handle().append(CommandRecord {
                    timestamp: Utc::now(),
                    machine: target,
                    path,
                    event: CommandEvent::Executed(result.clone()),
                });

                result
                    .map_err(MachineExecuteCommandError::ExecuteError)
                    .map_err(RuntimeRequestError::MachineExecuteCommand)
            }

            RuntimeRequestKind::SubscribeMachine {
                provider,
                subscriber,
            } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    return Err(MachineSubscribeError::ProviderNotFound)?;
                }

                // --- find subscriber ---
                let Some(instance) = find_machine(&mut self.machines, subscriber) else {
                    return Err(MachineSubscribeError::SubscriberNotFound)?;
                };

                if instance.subscriptions.contains_key(&provider) {
                    return Err(MachineSubscribeError::AlreadySubscribed)?;
                }

                let token_provider = LifetimeTokenOwner::new();

                // --- let machine subscribe to resources ---
                let mut ctx = SubscribeContext {
                    token: token_provider.new_token(),
                    provider,
                    resources: &mut self.resources,
                };

                // --- allow machine to handle subscription ---
                instance.machine.subscribe(&mut ctx)?;

                self.report.events.push(RuntimeEvent::SubscriptionAdded {
                    provider,
                    subscriber,

                    // TODO: record resources
                    resources: Default::default(),
                });

                Ok(())
            }

            RuntimeRequestKind::UnsubscribeMachine {
                provider,
                subscriber,
            } => {
                // --- find subscriber ---
                let Some(machine) = find_machine(&mut self.machines, subscriber) else {
                    return Err(MachineUnsubscribeError::SubscriptionNotFound)?;
                };

                // --- remove entry if present ---
                if machine.subscriptions.remove(&provider).is_some() {
                    self.report.events.push(RuntimeEvent::SubscriptionRemoved {
                        provider,
                        subscriber,
                    });

                    Ok(())
                } else {
                    Err(MachineUnsubscribeError::SubscriptionNotFound)?
                }
            }
        }
    }
}
