const NAMES_COUNT_MAX: usize = 2048;
const NAME_LEN_MAX: usize = 96;

const CONFIG_PROPERTIES_REGISTRY_ID: usize = 1;
const CONFIG_PROPERTIES_COUNT_MAX: usize = 512;

const STATE_PROPERTIES_REGISTRY_ID: usize = 2;
const STATE_PROPERTIES_COUNT_MAX: usize = 512;

const MEASUREMENTS_COUNT_MAX: usize = 512;

mod types;
pub use types::Journal;
pub use types::JournalHandle;
pub use types::RegisterError;
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
pub use config_property::ConstrainedConfigProperty;
pub use config_property::ConfigPropertyManager;

pub type ConfigPropertyResolver<'a> = 
    PropertyResolver<'a, CONFIG_PROPERTIES_REGISTRY_ID, CONFIG_PROPERTIES_COUNT_MAX>;

pub type ConfigPropertyReader<'a> = 
    PropertyReader<'a, CONFIG_PROPERTIES_REGISTRY_ID, CONFIG_PROPERTIES_COUNT_MAX>;

pub type ConfigPropertyAccessHandle<T> = PropertyAccessHandle<CONFIG_PROPERTIES_REGISTRY_ID, T>;

// --- state properties ---
mod state_property;
pub use state_property::StateProperty;
pub use state_property::StatePropertyManager;

pub type StatePropertyResolver<'a> = 
    PropertyResolver<'a, STATE_PROPERTIES_REGISTRY_ID, STATE_PROPERTIES_COUNT_MAX>;

pub type StatePropertyReader<'a> = 
    PropertyReader<'a, STATE_PROPERTIES_REGISTRY_ID, STATE_PROPERTIES_COUNT_MAX>;

pub type StatePropertyAccessHandle<T> = PropertyAccessHandle<STATE_PROPERTIES_REGISTRY_ID, T>;

// --- measurements ---
mod measurement;
pub use measurement::MeasurementRegistry;
pub use measurement::MeasurementResolver;
pub use measurement::MeasurementReader;

// --- name registry ---
#[derive(Debug, Default)]
pub struct NameRegistry(heapless::FnvIndexMap<&'static str, &'static str, NAMES_COUNT_MAX>);

impl NameRegistry {
    pub fn new() -> Self { Self(Default::default()) }

    /// Interns a name: returns a 'static lifetime version of input &str.
    /// Achieved by keeping a registry of all registered names, which are
    /// behind the scenes leaked strings. Bounded by the vec limit, 
    /// so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    pub fn register_name(&mut self, name: &str) -> Result<&'static str, RegisterError> {
        let reg = &mut self.0;

        if name.len() > NAME_LEN_MAX {
            return Err(RegisterError::NameTooLarge { name: name.to_string() });
        }

        if let Some(&existing) = reg.get(name) {
            return Ok(existing);
        }

        if reg.len() >= reg.capacity() {
            return Err(RegisterError::NameRegistryFull { name: name.to_string() })
        }

        // entry not found, create a new one by leaking the address
        let leaked: &'static str = name.to_string().leak();
        reg.insert(leaked, leaked).expect("validated prior");
        Ok(leaked)
    }
}

