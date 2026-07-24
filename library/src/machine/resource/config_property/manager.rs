use std::cell::Ref;

use qitech_framework_common::MachineConfigMutation;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::ScalarValue;

use super::ConfigProperty;
use crate::machine::resource::Journal;
use crate::machine::resource::JournalHandle;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyReader;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::config_property::ApiWriteConfigError;
use crate::machine::resource::config_property::WriteConfigError;
use crate::machine::resource::error::RegisterError;
use crate::machine::resource::kind;
use crate::machine::resource::property::ExtractFn;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;
type Kind = kind::ConfigProperty;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, RegistryMetadata>;

pub type Resolver<'a> =
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, RegistryMetadata>;

pub type Reader<'a> = PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, RegistryMetadata>;

pub type ReaderHandle<T> = PropertyReadHandle<Kind, T>;

pub type WriteApiFn = Box<
    dyn Fn(
        u64,
        &mut Journal<MachineConfigMutation>,
        *mut u8,
        &str,
    ) -> Result<(), ApiWriteConfigError>,
>;

pub type ValidateAndRecord<T> =
    Box<dyn Fn(&mut JournalHandle<MachineConfigMutation>, &T) -> Result<(), WriteConfigError>>;

pub struct RegistryMetadata {
    write_api: WriteApiFn,
    extract_value: ExtractFn<ScalarValue>,
}

pub struct Manager {
    registry: Registry,
    journal: Journal<MachineConfigMutation>,
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
        let metadata = RegistryMetadata {
            write_api,
            extract_value,
        };

        let handle = self
            .registry
            .register::<T>(ident, name, "", extract_value, metadata)?;
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
