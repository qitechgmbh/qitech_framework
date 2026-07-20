pub mod types;
pub use types::RegisterError;
pub use types::ResolveError;
pub use types::ReadError;

mod property;
pub use property::PropertyRegistry;
pub use property::PropertyHandle;
pub use property::PropertyResolver;
pub use property::PropertyReader;
pub use property::PropertyReaderHandle;

mod measurement;
pub use measurement::MeasurementRegistry;
pub use measurement::MeasurementHandle;
pub use measurement::MeasurementResolver;
pub use measurement::MeasurementReader;
pub use measurement::MeasurementReaderHandle;

const NAMES_COUNT_MAX: usize = 2048;
const NAME_LEN_MAX: usize = 96;

const MEASUREMENTS_REGISTRY_ID: usize = 0;
const MEASUREMENTS_COUNT_MAX: usize = 512;

const CONFIG_PROPERTIES_REGISTRY_ID: usize = 1;
const CONFIG_PROPERTIES_COUNT_MAX: usize = 512;

const STATE_PROPERTIES_REGISTRY_ID: usize = 2;
const STATE_PROPERTIES_COUNT_MAX: usize = 512;

// --- name registry ---
#[derive(Debug, Default)]
pub struct NameRegistry(heapless::FnvIndexMap<&'static str, &'static str, NAMES_COUNT_MAX>);

impl NameRegistry {
    /// Interns a name: returns a 'static lifetime version of input &str.
    /// Achieved by keeping a registry of all registered names, which are
    /// behind the scenes leaked strings. Bounded by the vec limit, 
    /// so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    fn register_name(&mut self, name: &str) -> Result<&'static str, RegisterError> {
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

