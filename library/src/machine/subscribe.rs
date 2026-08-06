use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::request::SubscribeError;
use qitech_framework_core::with_uom_quantities;

use crate::resource::CachedPropertyView;
use crate::resource::Resources;
use crate::resource::SubscriptionToken;

pub type SubscribeResult<T> = Result<T, SubscribeError>;

// --- property ---
pub struct SubscribedProperty<T: Clone> {
    view: CachedPropertyView<T>,
    token: Weak<SubscriptionToken>,
}

impl<T: Clone> SubscribedProperty<T> {
    pub fn get_ref(&self) -> &T {
        _ = self.token.upgrade().expect("Subscription expired");

        self.view.read()
    }
}

impl<T: Copy> SubscribedProperty<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl SubscribedProperty<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }
        }

        impl SubscribedProperty<Option<$quantity>> {
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
    token: Rc<SubscriptionToken>,
    provider: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub(crate) fn new(
        provider: MachineIdentificationUnique,
        resources: &'a mut Resources,
        token: Rc<SubscriptionToken>,
    ) -> Self {
        Self {
            provider,
            resources,
            token,
        }
    }

    pub fn provider(&self) -> MachineIdentificationUnique {
        self.provider
    }

    /*
    pub fn subscribe_config_property<T: 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {

    }

    pub fn subscribe_state_property<T: 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
    }
    */

    pub fn subscribe_measurement<T: Clone + 'static>(
        &mut self,
        resource: &'static str,
    ) -> SubscribeResult<SubscribedProperty<T>> {
        let view = self
            .resources
            .measurements
            .new_cached_view(self.provider, resource)
            .unwrap();

        let prop = SubscribedProperty {
            view,
            token: Rc::downgrade(&self.token),
        };

        Ok(prop)
    }
}
