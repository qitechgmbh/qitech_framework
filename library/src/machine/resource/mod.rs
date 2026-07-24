use core::fmt;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use qitech_framework_common::MachineIdentificationUnique;

pub mod error;

mod property;
pub use property::PropertyAccessor;
pub use property::PropertyHandle;
pub use property::PropertyReadHandle;
pub use property::PropertyRegistry;
pub use property::PropertyResolver;

mod config_property;
pub use config_property::Accessor as ConfigPropertyReader;
pub use config_property::Manager as ConfigPropertyManager;
pub use config_property::RemoteHandle as ConfigPropertyReaderHandle;
pub use config_property::Resolver as ConfigPropertyResolver;

mod measurement;
pub use measurement::Reader as MeasurementReader;
pub use measurement::ReaderHandle as MeasurementAccessHandle;
pub use measurement::Registrar as MeasurementRegistrar;
pub use measurement::Resolver as MeasurementResolver;

mod state_property;
pub use state_property::Manager as StatePropertyManager;
pub use state_property::Reader as StatePropertyReader;
pub use state_property::ReaderHandle as StatePropertyReaderHandle;
pub use state_property::Resolver as StatePropertyResolver;

mod command;
pub use command::Handle as CommandHandle;
pub use command::Manager as CommandManager;

mod event;

mod conversion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: &'a str,
    postfix: &'a str,
}

// --- kind ---
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

trait KindVariant: kind::private::Sealed {}

mod kind {
    use super::*;

    pub mod private {
        pub trait Sealed {
            const KIND: super::Kind;
        }
    }

    #[derive(Debug)]
    pub struct StateProperty;

    #[derive(Debug)]
    pub struct ConfigProperty;

    #[derive(Debug)]
    pub struct Measurement;

    #[derive(Debug)]
    pub struct Command;

    #[derive(Debug)]
    pub struct Event;

    macro_rules! impl_kind {
        ($kind:tt) => {
            impl private::Sealed for $kind {
                const KIND: Kind = Kind::$kind;
            }

            impl KindVariant for $kind {}
        };
    }

    impl_kind!(ConfigProperty);
    impl_kind!(StateProperty);
    impl_kind!(Measurement);
    impl_kind!(Command);
    impl_kind!(Event);
}

impl fmt::Display for Kind {
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
pub type JournalBuffer<T> = heapless::Vec<T, 1024>;

#[derive(Debug, Default)]
pub struct Journal<T> {
    buffer: Rc<RefCell<JournalBuffer<T>>>,
}

impl<T> Journal<T> {
    fn init_handle(&self) -> JournalHandle<T> {
        JournalHandle {
            buffer: self.buffer.clone(),
        }
    }

    fn drain_with(&mut self, mut f: impl FnMut(&T)) {
        for entry in self.buffer.borrow_mut().drain(..) {
            f(&entry);
        }
    }
}

#[derive(Debug)]
pub struct JournalHandle<T> {
    buffer: Rc<RefCell<JournalBuffer<T>>>,
}

impl<T: Debug> JournalHandle<T> {
    fn append(&self, entry: T) -> Result<(), JournalAppendError> {
        self.buffer
            .borrow_mut()
            .push(entry)
            .map_err(|_| JournalAppendError)
    }
}

// --- access ---
#[derive(Debug)]
pub struct GrantLevel {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

// --- errors ---
#[derive(Debug)]
pub struct JournalAppendError;

impl core::fmt::Display for JournalAppendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("journal buffer is full")
    }
}

impl core::error::Error for JournalAppendError {}
