use std::rc::Rc;
use std::cell::{Ref, RefCell};
use control_core::{MachineConfigMutation, MachineIdentificationUnique, ScalarValue};

use crate::conversion::{Bounded, ScalarPropertyType};
use crate::resource::config_property::ConfigPropertyOptions;
use crate::resource::{
    Journal, JournalHandle, PropertyAccessHandle, PropertyReader,
    PropertyRegistry, PropertyResolver, RegisterError, kind,
};

use super::ConfigProperty;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry =
    PropertyRegistry<SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyResolver<'a> =
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyReader<'a> =
    PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyAccessHandle<T> = PropertyAccessHandle<kind::ConfigProperty, T>;

pub struct ConfigPropertyManager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl ConfigPropertyManager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        options: ConfigPropertyOptions<T::Value>,
    ) -> Result<ConfigProperty<T>, RegisterError>
    where
        T: ScalarPropertyType + 'static,
        T::Value: Bounded
    {
        let handle = self.registry.register::<T>(ident, name, "")?;
        handle.write(options.default_value.clone());

        let journal = JournalHandle::new(self.journal.clone());

        Ok(ConfigProperty {
            handle,
            journal,
            ident,
            name,
            default: options.default_value,
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
