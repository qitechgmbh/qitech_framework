use std::rc::Rc;
use std::cell::{Ref, RefCell};
use qitech_framework_common::{MachineConfigMutation, MachineIdentificationUnique, OperationResult, ScalarValue};

use crate::machine::error::BoundsError;
use crate::machine::resource::config_property::{ApiWriteConfigError, WriteConfigError};
use crate::machine::resource::property::ExtractFn;
use crate::machine::resource::{
    Journal, JournalHandle, PropertyReadHandle, PropertyReader,
    PropertyRegistry, PropertyResolver, kind,
};

use crate::machine::resource::error::RegisterError;
use super::ConfigProperty;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry =
    PropertyRegistry<SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue, RegistryMetadata>;

pub type Resolver<'a> =
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue, RegistryMetadata>;

pub type Reader<'a> =
    PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue, RegistryMetadata>;

pub type AccessHandle<T> = PropertyReadHandle<kind::ConfigProperty, T>;


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
pub type WriteApiFn = Box<dyn Fn(
    u64, Rc<RefCell<Journal<MachineConfigMutation>>>, *mut u8, &str) 
    -> Result<(), ApiWriteConfigError>
>;

pub type ValidateAndRecord<T> = Box<dyn Fn(&mut JournalHandle<MachineConfigMutation>, &T) -> Result<(), WriteConfigError>>;

pub struct RegistryMetadata {
    write_api: WriteApiFn,
    extract_value: ExtractFn<ScalarValue>,
}

pub struct Manager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl Manager {
    // pub fn handle_api_request(&mut self, request: SetMachineConfigurationRequest) {
// 
    // }

    pub fn register<T: Clone + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        default: T,
        write_api: WriteApiFn,
        validate_and_record: ValidateAndRecord<T>,
        extract_value: ExtractFn<ScalarValue>,
    ) -> Result<ConfigProperty<T>, RegisterError> {
        let metadata = RegistryMetadata { write_api, extract_value };

        let handle = self.registry.register::<T>(ident, name, "", extract_value, metadata)?;
        handle.write(default.clone());

        let journal = JournalHandle::new(self.journal.clone());
        Ok(ConfigProperty {
            handle,
            journal,
            validate: validate_and_record,
            default,
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
