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

#[derive(Debug, Default)]
pub struct ResourceJournals {
    config: ResourceJournal<MachineConfigMutation>,
    state:  ResourceJournal<MachineStateMutation>,
    event:  ResourceJournal<MachineEvent>,
    logs:   ResourceJournal<LogRecord>,
}

impl ResourceJournals {
    pub fn new() -> Self { Self::default() }
}

pub enum ResourceRegisterError {
    NameTooLarge { 
        name: String,
    },
    NameRegistryFull {
        name: String 
    },
    RegistryFull { 
        name: &'static str 
    },
    AlreadyRegistered { 
        name: &'static str 
    },
    TypeTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
    AlignmentTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceResolveError {
    NoSuchProperty,
    InvalidType,
}

#[derive(Debug)]
pub struct ResourceReadError;

impl fmt::Display for ResourceReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expired handle")
    }
}

impl std::error::Error for ResourceReadError {}