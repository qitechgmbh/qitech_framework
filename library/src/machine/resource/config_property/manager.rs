use std::rc::Rc;
use std::cell::{Ref, RefCell};
use qitech_framework_common::{MachineConfigMutation, MachineIdentificationUnique, ScalarValue};

use crate::machine::error::BoundsError;
use crate::machine::resource::property::Extract;
use crate::machine::resource::{
    Journal, JournalHandle, PropertyAccessHandle, PropertyReader,
    PropertyRegistry, PropertyResolver, kind,
};

use crate::machine::resource::error::RegisterError;
use super::ConfigProperty;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry =
    PropertyRegistry<SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type Resolver<'a> =
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type Reader<'a> =
    PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type AccessHandle<T> = PropertyAccessHandle<kind::ConfigProperty, T>;


// store property metadata directly in the manager
// expose handle to property

// user: ConfigProperty<Length> | write() / read()

// what does manager need: export from raw bytes / validate from Scalar Value / write from Scalar Value
struct Entry {
    // what if both 
}

// api mutate also goes through journal -> on write() require ScalarValue

// api mutate directly

// commands -> easy we just receive serialized data / events -> easy since we dont do shit

// requires: validate when a api request comes in aka scalar value
enum RegistryMetadata {
    Enum(RegistryMetadataAny<i64, i64>),
    String(RegistryMetadataAny<String, u64>),
    Boolean(RegistryMetadataAny<bool>),
    Integer(RegistryMetadataAny<i64>),
    Float(RegistryMetadataAny<i64>),
}

struct RegistryMetadataAny<T, B> {
    min: Option<B>,
    max: Option<B>,

    validate_bounds: Option<fn(&T) -> Result<(), BoundsError>>,
    validate_custom: Option<fn(&T) -> Result<(), String>>,
    append_journal: fn(&mut JournalHandle<MachineConfigMutation>, T),
    extract_value: Extract<ScalarValue>,
    validate_api_value: Option<Box<>>,
}

pub struct Manager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl Manager {
    pub fn handle_api_request(&mut self, request: SetMachineConfigurationRequest) {

    }

    pub fn create<T: Clone + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        default: T,
        validate_bounds: Option<fn(&T) -> Result<(), BoundsError>>,
        validate_custom: Option<fn(&T) -> Result<(), String>>,
        append_journal: fn(&mut JournalHandle<MachineConfigMutation>, T),
        extract_value: Extract<ScalarValue>,
    ) -> Result<ConfigProperty<T>, RegisterError> {
        let handle = self.registry.register::<T>(ident, name, "", extract_value)?;
        handle.write(default.clone());

        let journal = JournalHandle::new(self.journal.clone());

        Ok(ConfigProperty {
            handle,
            journal,
            ident,
            resource_path: name,
            default,
            pred: None,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub fn journal(&self) -> Ref<'_, Journal<MachineConfigMutation>> {
        self.journal.borrow()
    }

    pub fn clear_journal(&self) {
        self.journal.borrow_mut().clear();
    }
}
