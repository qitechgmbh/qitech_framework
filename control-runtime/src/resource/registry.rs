use control_core::MachineIdentificationUnique;

use crate::resource::{
    ResourceRegisterError,
    MachineConfigPropertyHandle, 
    MachineConfigPropertyRegistry, 
    MachineMeasurementHandle, 
    MachineMeasurementRegistry, 
    MachineStatePropertyRegistry
};

const NAMES_COUNT_MAX: usize = 2048;
const NAME_LEN_MAX: usize = 96;

#[derive(Debug, Default)]
pub struct NameRegistry(heapless::FnvIndexMap<&'static str, &'static str, NAMES_COUNT_MAX>);

impl NameRegistry {
    /// Interns a name: returns a 'static lifetime version of input &str.
    /// Achieved by keeping a registry of all registered names, which are
    /// behind the scenes leaked strings. Bounded by the vec limit, 
    /// so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    fn register_name(&mut self, name: &str) -> Result<&'static str, ResourceRegisterError> {
        let reg = &mut self.0;

        if name.len() > NAME_LEN_MAX {
            return Err(ResourceRegisterError::NameTooLarge { name: name.to_string() });
        }

        if let Some(&existing) = reg.get(name) {
            return Ok(existing);
        }

        if reg.len() >= reg.capacity() {
            return Err(ResourceRegisterError::NameRegistryFull { name: name.to_string() })
        }

        // entry not found, create a new one by leaking the address
        let leaked: &'static str = name.to_string().leak();
        reg.insert(leaked, leaked);
        Ok(leaked)
    }
}

#[derive(Debug)]
pub struct MachineResourceRegistry {
    names:       NameRegistry,
    config:      MachineConfigPropertyRegistry,
    state:       MachineStatePropertyRegistry,
    measurement: MachineMeasurementRegistry,
}

impl MachineResourceRegistry {
    pub(crate) fn new() -> Self {
        Self {
            names: Default::default(),
            config: MachineConfigPropertyRegistry::new(),
            state: MachineStatePropertyRegistry::new(),
            measurement: MachineMeasurementRegistry::new(),
        }
    }

    pub(crate) fn register_config_property<T: 'static>(
        &mut self,         
        ident: MachineIdentificationUnique,
        name: &str
    ) -> Result<MachineConfigPropertyHandle<T>, ResourceRegisterError> {
        let name = self.register_name(name)?;
        self.config.register(ident, name)
    }

    pub(crate) fn register_state_property<T: 'static>(
        &mut self,         
        ident: MachineIdentificationUnique,
        name: &str
    ) -> Result<MachineStatePropertyHandle<T>, ResourceRegisterError> {
        let name = self.register_name(name)?;
        self.state.register(ident, name)
    }

    pub(crate) fn register_measurement<T: 'static>(
        &mut self,         
        ident: MachineIdentificationUnique,
        name: &str
    ) -> Result<MachineMeasurementHandle, ResourceRegisterError> {
        let name = self.register_name(name)?;
        self.measurement.register(ident, name)
    }

    // frees up all resources associated with the provided machine identifier
    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.config.unregister_machine(ident);
        self.state.unregister_machine(ident);
        self.measurement.unregister_machine(ident);
    }

    /// Interns a name: returns a 'static lifetime version of input &str.
    /// Achieved by keeping a registry of all registered names, which are
    /// behind the scenes leaked strings. Bounded by the vec limit, 
    /// so worst case is ~0.2 MiB (2048 * 96). 
    /// Avoids reallocating on every clone without multi-threading issues.
    fn register_name(&mut self, name: &str) -> Result<&'static str, ResourceRegisterError> {
        if name.len() > NAME_LEN_MAX {
            return Err(ResourceRegisterError::NameTooLarge { name: name.to_string() });
        }

        if let Some(&existing) = self.names.get(name) {
            return Ok(existing);
        }

        if self.names.len() >= self.names.capacity() {
            return Err(ResourceRegisterError::NameRegistryFull { name: name.to_string() })
        }

        // entry not found, create a new one by leaking the address
        let leaked: &'static str = name.to_string().leak();
        self.names.insert(leaked, leaked);
        Ok(leaked)
    }
}
