use std::borrow::Cow;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_core::MachineIdentificationUnique;
use qitech_framework_core::with_uom_quantities;
use thiserror::Error;

pub struct SubscriptionToken;

// --- property ---
pub struct SubscribedProperty<T> {
    token: Weak<SubscriptionToken>,
    p_value: NonNull<T>,
}

impl<T> SubscribedProperty<T> {
    pub fn new(token: Weak<SubscriptionToken>, p_value: NonNull<T>) -> Self {
        Self { token, p_value }
    }

    pub fn get_ref(&self) -> &T {
        self.token
            .upgrade()
            .expect("Subscriber outlived subscription");
        unsafe { self.p_value.as_ref() }
    }
}

impl<T: Copy> SubscribedProperty<T> {
    pub fn get(&self) -> T {
        self.token
            .upgrade()
            .expect("Subscriber outlived subscription");

        unsafe { self.p_value.read() }
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

// --- registry ---
#[derive(Default)]
pub struct SubscriptionRegistry {
    inner: HashMap<SubscriptionEntry, Rc<SubscriptionToken>>,
}

impl SubscriptionRegistry {
    pub fn register(
        &mut self,
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
        resource: &'static str,
    ) -> Result<Weak<SubscriptionToken>, SubscribeError> {
        let key = SubscriptionEntry {
            producer,
            consumer,
            resource: Cow::Borrowed(resource),
        };

        if self.inner.contains_key(&key) {
            return Err(SubscribeError::Duplicate);
        }

        let token = Rc::new(SubscriptionToken);
        self.inner.insert(key, token.clone());
        Ok(Rc::downgrade(&token))
    }

    // e.g. winder , need to clear all its subscriptions
    pub fn unregister(
        &mut self,
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) {
        self.inner
            .retain(|key, _| !(key.producer == producer && key.consumer == consumer));
    }

    // e.g. laser disconnected and need to clear all its subscriptions and subscriber
    pub fn unregister_producer(&mut self, producer: MachineIdentificationUnique) {
        self.inner
            .retain(|key, _| key.producer != producer && key.consumer != producer);
    }

    // iterate all resources subscribed by `consumer`
    pub fn subscribed_resources<'a>(
        &'a self,
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) -> impl Iterator<Item = &'a str> + 'a {
        self.inner
            .keys()
            .filter(move |entry| entry.producer == producer && entry.consumer == consumer)
            .map(|entry| entry.resource.as_ref())
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct SubscriptionEntry {
    producer: MachineIdentificationUnique,
    consumer: MachineIdentificationUnique,
    resource: Cow<'static, str>,
}

// --- error ---
#[derive(Error, Debug, Clone, Copy)]
pub enum SubscribeError {
    #[error("Machine is already subscribed to that resource")]
    Duplicate,

    #[error("Type does not match the registry entry")]
    InvalidType,

    #[error("Property does not exist on Machine")]
    NoSuchProperty,
}
