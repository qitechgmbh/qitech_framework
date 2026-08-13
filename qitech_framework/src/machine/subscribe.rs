use std::ptr::NonNull;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::request::MachineSubscribeError;
use qitech_framework_core::with_uom_quantities;

use crate::resource::LifetimeToken;
use crate::resource::ResourceRegistry;

/// A handle to a property belonging to a subscribed machine.
///
/// The property is only valid while the associated subscription is alive.
/// Accessing the property after the subscription has ended will panic.
pub struct RemoteProperty<T> {
    p_value: NonNull<T>,
    token: LifetimeToken,
}

impl<T> RemoteProperty<T> {
    pub fn get_ref(&self) -> &T {
        self.token.validate();
        unsafe { self.p_value.as_ref() }
    }
}

impl<T: Copy> RemoteProperty<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl RemoteProperty<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }
        }

        impl RemoteProperty<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }
        }
    };
}

with_uom_quantities!(impl_uom);

// --- context ---
pub struct SubscribeContext<'a> {
    pub(crate) token: LifetimeToken,
    pub(crate) provider: MachineIdentificationUnique,
    pub(crate) resources: &'a ResourceRegistry,
}

impl<'a> SubscribeContext<'a> {
    pub const fn provider(&self) -> MachineIdentificationUnique {
        self.provider
    }

    pub fn config<T: 'static>(
        &self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, MachineSubscribeError> {
        let value = self
            .resources
            .config_properties
            .get_cached(self.provider, resource)?;

        Ok(RemoteProperty {
            token: self.token.clone(),
            p_value: value,
        })
    }

    pub fn state<T: Clone + 'static>(
        &self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, MachineSubscribeError> {
        let value = self
            .resources
            .state_properties
            .get_cached(self.provider, resource)?;

        Ok(RemoteProperty {
            p_value: value,
            token: self.token.clone(),
        })
    }

    pub fn measurement<T: Clone + 'static>(
        &self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, MachineSubscribeError> {
        let value = self
            .resources
            .measurements
            .get_cached(self.provider, resource)?;

        Ok(RemoteProperty {
            p_value: value,
            token: self.token.clone(),
        })
    }
}
