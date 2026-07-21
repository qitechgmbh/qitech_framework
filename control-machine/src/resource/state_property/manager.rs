use std::cell::{Ref, RefCell};
use std::rc::Rc;
use control_core::{MachineIdentificationUnique, MachineStateMutation, ScalarValue};

use crate::conversion::{PropertyType, ScalarPropertyType};
use crate::resource::{
    RegisterError,
    kind,
    Journal, 
    JournalHandle, 
    StatePropertySpecification,
    PropertyRegistry, 
    PropertyResolver, 
    PropertyReader,
    PropertyAccessHandle, 
};
use super::StateProperty;

use crate::resource::REGISTRY_ID_STATE_PROPERTIES;
const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    REGISTRY_ID_STATE_PROPERTIES, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyResolver<'a> = PropertyResolver<
    'a, 
    REGISTRY_ID_STATE_PROPERTIES, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyReader<'a> = PropertyReader<
    'a, 
    REGISTRY_ID_STATE_PROPERTIES, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyAccessHandle<T> = PropertyAccessHandle<REGISTRY_ID_STATE_PROPERTIES, T>;

pub struct StatePropertyManager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineStateMutation>>>,
}

impl StatePropertyManager {
    pub fn register<Spec>(
        &mut self,
        ident: MachineIdentificationUnique,
        initial_value: <Spec::Type as PropertyType>::Value,
    ) -> Result<StateProperty<Spec::Type>, RegisterError> 
    where 
        Spec: StatePropertySpecification + 'static,
        Spec::Type: ScalarPropertyType,
    {
        let handle = self.registry.register::<Spec>(ident)?;
        handle.write(initial_value);

        let journal = JournalHandle::new(self.journal.clone());

        Ok(StateProperty {
            handle,
            journal,
            ident,
            name: Spec::NAME,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub fn journal(&self) -> Ref<'_, Journal<MachineStateMutation>> {
        self.journal.borrow()
    }

    pub fn clear_journal(&self) {
        self.journal.borrow_mut().clear();
    }
}
