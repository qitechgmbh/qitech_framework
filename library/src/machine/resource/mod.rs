use core::fmt;
use std::borrow::Cow;
use std::fmt::Debug;

use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::report::ParameterConstraints as ConfigPropertyWriteConstraints;
pub use qitech_framework_core::report::MachineConfigWriteCapability as ConfigPropertyWriteCapability;

pub mod error;

mod property;
use property::PropertyHandle;
use property::PropertyRegistry;

mod config_property;
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyCapabilities;
pub use config_property::ConfigPropertyCapabilitiesAny;
pub use config_property::GetCapabilitiesFn;
pub use config_property::IntoGetCapabilitiesFn;
pub use config_property::IntoOnChangedFn;
pub use config_property::Manager as ConfigPropertyManager;
pub use config_property::OnChangedFn;

mod measurement;
pub use measurement::Manager as MeasurementManager;
pub use measurement::Measurement;
pub use measurement::RegisterOptions as MeasurementRegisterOptions;

mod state_property;
pub use state_property::StateProperty;
pub use state_property::StatePropertyRegistry as StatePropertyManager;

mod command;
pub use command::CanExecuteFn;
pub use command::CommandRegistry as CommandManager;
pub use command::ExecuteFn;
pub use command::IntoCanExecuteFn;
pub use command::IntoExecuteFn;

mod event;
pub use event::Emitter as EventEmitter;
pub use event::Manager as EventManager;
pub use event::SubscribedEvent;

pub mod constraints;
pub use constraints::EnumConfigPropertyConstraints;
pub(crate) mod conversion;
pub(crate) mod subscription;

// --- key ---
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
