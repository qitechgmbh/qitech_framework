use qitech_framework_core::ident::MachineIdentificationUnique;
use serde::de::DeserializeOwned;

use crate::machine::Resources;
use crate::machine::SubscribedEvent;
use crate::machine::error::SubscribeResult;
use crate::machine::resource::subscription::SubscribedProperty;

pub struct SubscribeContext<'a> {
    provider: MachineIdentificationUnique,
    subscriber: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub(crate) fn new(
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
        resources: &'a mut Resources,
    ) -> Self {
        Self {
            provider,
            subscriber,
            resources,
        }
    }

    pub fn producer(&self) -> MachineIdentificationUnique {
        self.provider
    }

    pub fn subscribe_config_property<T: 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        Ok(self.resources.config_properties.create_subscriber::<T>(
            self.provider,
            self.subscriber,
            resource,
        )?)
    }

    pub fn subscribe_state_property<T: 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        Ok(self.resources.state_properties.create_subscriber::<T>(
            self.provider,
            self.subscriber,
            resource,
        )?)
    }

    pub fn subscribe_measurement<T: 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        Ok(self.resources.measurements.create_subscriber::<T>(
            self.provider,
            self.subscriber,
            resource,
        )?)
    }

    pub fn subscribe_event<T>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedEvent<T>>
    where
        T: DeserializeOwned + 'static,
    {
        Ok(self.resources.events.create_subscriber::<T>(
            self.provider,
            self.subscriber,
            resource,
        )?)
    }
}
