use control_core::MachineResourceKind;

const REGISTRY_ID_CONFIG_PROPERTIES: usize = 1;
const REGISTRY_ID_STATE_PROPERTIES: usize = 2;
const REGISTRY_ID_MEASUREMENTS: usize = 3;

pub trait Kind: private::Kind {}
mod private { pub trait Kind: { const KIND: control_core::MachineResourceKind; }}

pub mod kind {
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
}

macro_rules! impl_kind {
    ($kind:tt) => {
        impl private::Kind for kind::$kind {
            const KIND: MachineResourceKind = MachineResourceKind::$kind;
        }
        
        impl Kind for kind::$kind {}
    };
}

impl_kind!(ConfigProperty);
impl_kind!(StateProperty);
impl_kind!(Measurement);
impl_kind!(Command);
impl_kind!(Event);

mod types;
pub use types::Journal;
pub use types::JournalHandle;
pub use types::RegisterError;
pub use types::RegisterErrorReason;
pub use types::ResolveError;
pub use types::ReadError;

mod property;
pub use property::PropertyRegistry;
pub use property::PropertyHandle;
pub use property::PropertyResolver;
pub use property::PropertyReader;
pub use property::PropertyAccessHandle;
// --- config properties ---
mod config_property;
pub use config_property::ConfigPropertySpecification;
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyManager;
pub use config_property::ConfigPropertyResolver;
pub use config_property::ConfigPropertyReader;
pub use config_property::ConfigPropertyAccessHandle;

// --- state properties ---
mod state_property;
pub use state_property::StateProperty;
pub use state_property::StatePropertySpecification;
pub use state_property::StatePropertyManager;
pub use state_property::StatePropertyResolver;
pub use state_property::StatePropertyReader;
pub use state_property::StatePropertyAccessHandle;

// --- measurements ---
mod measurement;
pub use measurement::Measurement;
pub use measurement::MeasurementSpecification;
pub use measurement::MeasurementManager;
pub use measurement::MeasurementResolver;
pub use measurement::MeasurementReader;
pub use measurement::MeasurementAccessHandle;

pub trait Specification {
    const NAME: &'static str;
    type Kind: Kind;
    type Type: 'static;
}

// config_property!("diameter.target") -> ctx.config_property("diameter.target", kind::)
// config_property!(name = "diameter.target", predicate = |x| x == 10)