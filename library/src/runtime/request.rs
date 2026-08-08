use std::rc::Rc;

use qitech_framework_core::report::RuntimeEvent;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::request::RuntimeResponse;
use qitech_framework_core::request::SubscribeError;
use qitech_framework_core::request::UnsubscribeError;
use qitech_framework_core::request::WriteConfigPropertyError;
use qitech_framework_core::request::WriteMachineDeviceInfoError;
use qitech_framework_core::session::RuntimeTransport;

use crate::Runtime;
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
                let Some(machine) = find_machine(&mut self.machines, target) else {
                    return Err(WriteConfigPropertyError::MachineNotFound)?;
                };

                

                // --- retrieve the context ---
                let context = self
                    .resources
                    .config_properties
                    .execute_context(target, &path);

                let Some(context) = context else {
                    return Err(WriteConfigPropertyError::ResourceNotFound)?;
                };

                // --- write the value ---
                let result = context.execute(machine, value.clone());

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

                self.journals.config_property.new_handle().append(record);

                // --- yield the result ---
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(WriteConfigPropertyError::WriteError(e))?,
                }
            }

            RuntimeRequestKind::InvokeMachineCommand {
                target,
                path: resource,
            } => {
                // let result = self
                //     .resources
                //     .commands
                //     .invoke(target, machine_ref, &resource)
                //     .unwrap();

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

            RuntimeRequestKind::SubscribeMachine {
                provider,
                subscriber,
            } => {
                // --- ensure provider exists ---
                if find_machine(&mut self.machines, provider).is_none() {
                    return Err(SubscribeError::ProviderNotFound)?;
                }

                // --- find subscriber ---
                let Some(machine) = find_machine(&mut self.machines, subscriber) else {
                    return Err(SubscribeError::SubscriberNotFound)?;
                };

                let duplicate = self
                    .subscriptions
                    .iter()
                    .any(|s| s.provider == provider && s.subscriber == subscriber);

                if duplicate {
                    return Err(SubscribeError::DuplicateSubscription)?;
                }

                let subscription = Subscription {
                    provider,
                    subscriber,
                    token: Rc::new(Default::default()),
                };

                // --- let machine subscribe to resources ---
                let ctx = SubscribeContext::new(
                    provider,
                    &mut self.resources,
                    subscription.token.clone(),
                );

                // --- allow machine to handle subscription ---
                machine.subscribe(ctx)?;

                // --- register subscription ---
                self.subscriptions
                    .push(subscription)
                    .expect("Exceeded global subscription limit");

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
                let Some(index) = self
                    .subscriptions
                    .iter()
                    .position(|s| s.provider == provider && s.subscriber == subscriber)
                else {
                    return Err(UnsubscribeError::SubscriptionNotFound)?;
                };

                self.subscriptions.remove(index);

                let machine = find_machine(&mut self.machines, subscriber)
                    .expect("No machine when subscription present");

                // --- tell machine that the subscription is not longer valid ---
                machine.unsubscribe(provider);

                self.report.events.push(RuntimeEvent::SubscriptionRemoved {
                    provider,
                    subscriber,
                });

                Ok(())
            }
        }
    }
}
