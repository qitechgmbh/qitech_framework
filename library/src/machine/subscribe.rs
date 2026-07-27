use qitech_framework_common::MachineIdentificationUnique;
use serde::de::DeserializeOwned;

use crate::machine::Resources;
use crate::machine::SubscribedEvent;
use crate::machine::error::SubscribeResult;
use crate::machine::resource::subscription::SubscribedProperty;

pub struct SubscribeContext<'a> {
    producer: MachineIdentificationUnique,
    consumer: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub(crate) fn new(
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
        resources: &'a mut Resources,
    ) -> Self {
        Self {
            producer,
            consumer,
            resources,
        }
    }

    pub fn producer(&self) -> MachineIdentificationUnique {
        self.producer
    }

    pub fn subscribe_config<T>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        // self.resources.config_properties
        //     .create_subscriber(self.producer, self.consumer, resource)?
        todo!()
    }

    pub fn subscribe_state<T>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        // self.resources.config_properties
        //     .create_subscriber(self.producer, self.consumer, resource)?
        todo!()
    }

    pub fn subscribe_measurement<T>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        // self.resources.config_properties
        //     .create_subscriber(self.producer, self.consumer, resource)?
        todo!()
    }

    pub fn subscribe_event<T>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedEvent<T>>
    where
        T: DeserializeOwned + 'static,
    {
        todo!()
    }
}
