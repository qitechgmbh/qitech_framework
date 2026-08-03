use core::fmt;
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

pub use qitech_framework_core::report::MachineConfigWriteCapability as ConfigPropertyWriteCapability;
use qitech_framework_core::ident::MachineIdentificationUnique;

pub mod error;

mod property;
use property::PropertyHandle;
use property::PropertyManager;

mod config_property;
pub use config_property::ConfigPropertyCapabilitiesAny;
pub use config_property::IntoGetCapabilitiesFn;
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyCapabilities;
pub use config_property::GetCapabilitiesFn;
pub use config_property::Manager as ConfigPropertyManager;

mod measurement;
pub use measurement::Manager as MeasurementManager;
pub use measurement::Measurement;
pub use measurement::RegisterOptions as MeasurementRegisterOptions;

mod state_property;
pub use state_property::Manager as StatePropertyManager;
pub use state_property::StateProperty;

mod command;
pub use command::CanExecuteFn;
pub use command::ExecuteFn;
pub use command::IntoCanExecuteFn;
pub use command::IntoExecuteFn;
pub use command::Manager as CommandManager;

mod event;
pub use event::Emitter as EventEmitter;
pub use event::Manager as EventManager;
pub use event::SubscribedEvent;

pub mod constraints;
pub(crate) mod conversion;
pub(crate) mod subscription;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: Cow<'a, str>,
}

impl<'a> Key<'a> {
    pub fn from_str(ident: MachineIdentificationUnique, path: &'a str) -> Self {
        Self {
            ident,
            path: Cow::Borrowed(path),
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

// my_command[]
// my_command["target_diameter"].set_bounds(0.0, 1.0)

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
        self.buffer.borrow_mut().push(entry);
    }
}
