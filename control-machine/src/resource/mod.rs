pub mod kind;
pub use kind::Kind;

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
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyOptions;
pub use config_property::ConfigPropertyManager;
pub use config_property::ConfigPropertyResolver;
pub use config_property::ConfigPropertyReader;
pub use config_property::ConfigPropertyAccessHandle;

// --- state properties ---
mod state_property;
pub use state_property::StateProperty;
pub use state_property::StatePropertyOptions;
pub use state_property::StatePropertyManager;
pub use state_property::StatePropertyResolver;
pub use state_property::StatePropertyReader;
pub use state_property::StatePropertyAccessHandle;

// --- measurements ---
mod measurement;
pub use measurement::Measurement;
pub use measurement::MeasurementOptions;
pub use measurement::MeasurementManager;
pub use measurement::MeasurementResolver;
pub use measurement::MeasurementReader;
pub use measurement::MeasurementAccessHandle;

// config_property!("diameter.target") -> ctx.config_property("diameter.target", kind::)
// config_property!(name = "diameter.target", predicate = |x| x == 10)