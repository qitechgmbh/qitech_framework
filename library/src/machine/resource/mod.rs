use core::fmt;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_common::MachineIdentificationUnique;

pub mod error;

mod property;
use property::PropertyHandle;
use property::PropertyManager;
use property::Subscriber;

mod config_property;
pub use config_property::ConfigProperty;
pub use config_property::Manager as ConfigPropertyManager;
pub use config_property::RegisterOptions as ConfigPropertyRegisterOptions;
pub use config_property::RemoteHandle as ConfigPropertyReaderHandle;

mod measurement;
pub use measurement::Manager as MeasurementManager;
pub use measurement::Measurement;
pub use measurement::RegisterOptions as MeasurementRegisterOptions;

mod state_property;
pub use state_property::Manager as StatePropertyManager;
pub use state_property::ReaderHandle as StatePropertyReaderHandle;
pub use state_property::StateProperty;

mod command;
pub use command::ExecuteError as CommandExecuteError;
pub use command::Handle as CommandHandle;
pub use command::Manager as CommandManager;
pub use command::RegisterOptions as CommandRegisterOptions;

mod event;
pub use event::Emitter as EventEmitter;
pub use event::Manager as EventManager;

use crate::machine::resource::error::SubscribeErrorKind;
use crate::machine::resource::error::SubscribeResult;

pub mod conversion;

mod registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: &'a str,
    postfix: &'a str,
}

impl<'a> Key<'a> {
    pub fn simple(ident: MachineIdentificationUnique, path: &'a str) -> Self {
        Self {
            ident,
            path,
            postfix: "",
        }
    }
}

// --- kind ---
#[derive(Debug, Clone, Copy)]
pub enum ResourceKind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

pub trait PropertyKind: property_kind::Sealed {}

mod property_kind {
    use super::*;

    pub trait Sealed {
        const RESOURCE_KIND: super::ResourceKind;
    }

    pub struct StateProperty;
    pub struct ConfigProperty;
    pub struct Measurement;

    macro_rules! impl_kind {
        ($kind:tt) => {
            impl Sealed for $kind {
                const RESOURCE_KIND: ResourceKind = ResourceKind::$kind;
            }

            impl PropertyKind for $kind {}
        };
    }

    impl_kind!(ConfigProperty);
    impl_kind!(StateProperty);
    impl_kind!(Measurement);
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConfigProperty => "config property",
            Self::StateProperty => "state property",
            Self::Measurement => "measurement",
            Self::Command => "command",
            Self::Event => "event",
        };
        f.write_str(s)
    }
}

// --- journal ---
#[derive(Debug)]
pub struct Journal<T> {
    buffer: Rc<RefCell<Vec<T>>>,
}

impl<T> Journal<T> {
    pub fn new() -> Self {
        Self {
            buffer: Default::default(),
        }
    }

    fn new_handle(&self) -> JournalHandle<T> {
        JournalHandle {
            buffer: self.buffer.clone(),
        }
    }

    fn drain_with(&mut self, mut f: impl FnMut(T)) {
        for entry in self.buffer.borrow_mut().drain(..) {
            f(entry);
        }
    }
}

// compiler is too stupid to implement derive for this... sigh
impl<T> Default for Journal<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct JournalHandle<T> {
    buffer: Rc<RefCell<Vec<T>>>,
}

impl<T: Debug> JournalHandle<T> {
    fn append(&self, entry: T) {
        self.buffer.borrow_mut().push(entry)
    }
}

pub struct ResourceKey {
    ident: MachineIdentificationUnique,
    path: Cow<'static, str>,
}

type ResourceId = u64;

// --- subscription ---
type SubscriptionId = u64;
struct SubscriptionToken;

#[derive(Default)]
pub struct SubscriptionRegistry {
    counter: u64,
    inner: HashMap<SubscriptionEntry, Rc<SubscriptionToken>>,
}

impl SubscriptionRegistry {
    pub fn register(
        &mut self, 
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
        resource: &'static str,
    ) -> SubscribeResult<Weak<SubscriptionToken>> {
        let key = SubscriptionEntry {
            producer,
            consumer,
            resource,
        };

        if self.inner.contains_key(&key) {
            return Err(SubscribeErrorKind::Duplicate);
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
        self.inner.retain(|key, _| !(key.producer == producer && key.consumer == consumer));
    }

    // e.g. laser disconnected and need to clear all its subscriptions and subscriber
    pub fn unregister_producer(&mut self, producer: MachineIdentificationUnique) {
        self.inner.retain(|key, _| key.producer != producer && key.consumer != producer);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct SubscriptionEntry {
    producer: MachineIdentificationUnique,
    consumer: MachineIdentificationUnique,
    resource: &'static str,
}
