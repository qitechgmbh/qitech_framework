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
pub use config_property::ConfigProperty;
pub use config_property::Manager as ConfigPropertyManager;
pub use config_property::RegisterOptions as ConfigPropertyRegisterOptions;
pub use config_property::RemoteHandle as ConfigPropertyReaderHandle;
pub use config_property::Resolver as ConfigPropertyResolver;

mod measurement;
pub use measurement::Manager as MeasurementManager;
pub use measurement::Measurement;
pub use measurement::Reader as MeasurementReader;
pub use measurement::ReaderHandle as MeasurementAccessHandle;
pub use measurement::RegisterOptions as MeasurementRegisterOptions;
pub use measurement::Registrar as MeasurementRegistrar;
pub use measurement::Resolver as MeasurementResolver;

mod state_property;
pub use state_property::Manager as StatePropertyManager;
pub use state_property::Reader as StatePropertyReader;
pub use state_property::ReaderHandle as StatePropertyReaderHandle;
pub use state_property::Registrar as StatePropertyRegistrar;
pub use state_property::Resolver as StatePropertyResolver;
pub use state_property::StateProperty;

mod command;
pub use command::ExecuteError as CommandExecuteError;
pub use command::Handle as CommandHandle;
pub use command::Manager as CommandManager;
pub use command::Registrar as CommandRegistrar;

mod event;
pub use event::Emitter as EventEmitter;
pub(crate) use event::Manager as EventManager;
pub use event::Registrar as EventRegistrar;

pub mod conversion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: &'a str,
    postfix: &'a str,
}

// --- kind ---
#[derive(Debug, Clone, Copy)]
pub(crate) enum Kind {
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

    pub struct StateProperty;
    pub struct ConfigProperty;
    pub struct Measurement;
    pub struct Command;
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
const JOURNAL_CAPACITY: usize = 16384;

pub type JournalBuffer<T> = heapless::Vec<T, JOURNAL_CAPACITY>;

#[derive(Debug, Default)]
pub struct Journal<T> {
    buffer: Rc<RefCell<JournalBuffer<T>>>,
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
    fn append(&self, entry: T) {
        assert!(
            self.buffer.borrow_mut().push(entry).is_ok(),
            "Runtime exceeded maximum journal entries ({}) in a report cycle",
            JOURNAL_CAPACITY,
        );
    }
}
