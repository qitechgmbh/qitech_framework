use std::cell::{Ref, RefCell};
use std::rc::Rc;
use control_core::{MachineIdentificationUnique, MachineStateMutation, ScalarValue};

use crate::conversion::ScalarPropertyType;
use crate::resource::{
    RegisterError,
    kind,
    Journal, 
    JournalHandle, 
    PropertyRegistry, 
    PropertyResolver, 
    PropertyReader,
    PropertyAccessHandle, 
};

use super::{StateProperty, StatePropertyOptions};

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyResolver<'a> = PropertyResolver<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyReader<'a> = PropertyReader<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type StatePropertyAccessHandle<T> = PropertyAccessHandle<kind::StateProperty, T>;

pub struct StatePropertyManager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineStateMutation>>>,
}

impl StatePropertyManager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        options: StatePropertyOptions<T::Value>,
    ) -> Result<StateProperty<T>, RegisterError> 
    where 
        T: ScalarPropertyType + 'static
    {
        let handle = self.registry.register::<T>(ident, name, "")?;
        handle.write(options.initial_value);

        let journal = JournalHandle::new(self.journal.clone());

        Ok(StateProperty {
            handle,
            journal,
            ident,
            name,
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
