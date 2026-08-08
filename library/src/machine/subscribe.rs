use std::ptr::NonNull;
use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::request::SubscribeError;
use qitech_framework_core::with_uom_quantities;

use crate::machine::LifetimeToken;
use crate::machine::ResourceRegistry;

// --- subscription ---
#[derive(Debug, Clone)]
pub struct Subscription {
    provider: MachineIdentificationUnique,
    subscriber: MachineIdentificationUnique,
    token: Rc<LifetimeToken>,
}

// --- property ---
pub struct RemoteProperty<T: Clone> {
    p_cache: NonNull<T>,
    token: Weak<LifetimeToken>,
}

impl<T: Clone> RemoteProperty<T> {
    pub fn get_ref(&self) -> &T {
        _ = self.token.upgrade().expect("Subscription expired");
        unsafe { self.p_cache.as_ref() }
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
    pub(crate) token: Rc<LifetimeToken>,
    pub(crate) provider: MachineIdentificationUnique,
    pub(crate) resources: &'a mut ResourceRegistry,
}

impl<'a> SubscribeContext<'a> {
    pub fn provider(&self) -> MachineIdentificationUnique {
        self.provider
    }

    pub fn config<T: Clone + 'static>(
        &mut self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, SubscribeError> {
        let p_cache = self
            .resources
            .config_properties
            .get_cached(self.provider, resource)
            .unwrap();

        let prop = RemoteProperty {
            token: Rc::downgrade(&self.token),
            p_cache,
        };

        Ok(prop)
    }

    pub fn state<T: Clone + 'static>(
        &mut self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, SubscribeError> {
        let view = self
            .resources
            .state_properties
            .get_cached(self.provider, resource)
            .unwrap();

        let prop = RemoteProperty {
            p_cache: view,
            token: Rc::downgrade(&self.token),
        };

        Ok(prop)
    }

    pub fn measurement<T: Clone + 'static>(
        &mut self,
        resource: &'static str,
    ) -> Result<RemoteProperty<T>, SubscribeError> {
        let view = self
            .resources
            .measurements
            .get_cached(self.provider, resource)
            .unwrap();

        let prop = RemoteProperty {
            p_cache: view,
            token: Rc::downgrade(&self.token),
        };

        Ok(prop)
    }
}
