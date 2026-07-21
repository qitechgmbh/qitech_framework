use std::rc::Rc;
use std::cell::{Ref, RefCell};
use control_core::{MachineConfigMutation, MachineIdentificationUnique, ScalarValue};

use crate::conversion::{Bounded, PropertyType, ScalarPropertyType};
use crate::resource::{
    ConfigPropertySpecification, Journal, JournalHandle, PropertyAccessHandle, PropertyReader,
    PropertyRegistry, PropertyResolver, RegisterError, kind,
};

use super::ConfigProperty;

use crate::resource::REGISTRY_ID_CONFIG_PROPERTIES as REGISTRY_ID;
const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry =
    PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyResolver<'a> =
    PropertyResolver<'a, REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyReader<'a> =
    PropertyReader<'a, REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, kind::StateProperty, ScalarValue>;

pub type ConfigPropertyAccessHandle<T> = PropertyAccessHandle<REGISTRY_ID, T>;

pub struct ConfigPropertyManager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl ConfigPropertyManager {
    pub fn register<Spec>(
        &mut self,
        ident: MachineIdentificationUnique,
    ) -> Result<ConfigProperty<Spec::Type>, RegisterError>
    where
        Spec: ConfigPropertySpecification + 'static,
        Spec::Type: ScalarPropertyType,
        <Spec::Type as PropertyType>::Value: Bounded,
    {
        let handle = self.registry.register::<Spec>(ident)?;
        handle.write(Spec::default_value());

        let journal = JournalHandle::new(self.journal.clone());

        Ok(ConfigProperty {
            handle,
            journal,
            ident,
            name: Spec::NAME,
            default: Spec::default_value(),
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
