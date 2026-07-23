use std::cell::{Ref, RefCell};
use std::rc::Rc;
use qitech_framework_common::{MachineIdentificationUnique, MachineStateMutation, ScalarValue};

use crate::machine::resource::property::Extract;
use crate::machine::resource::{
    Journal, 
    JournalHandle, 
    PropertyAccessHandle, 
    PropertyReader,
    PropertyRegistry, 
    PropertyResolver, 
    error::RegisterResult,
    kind,
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

pub struct Manager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineStateMutation>>>,
}

impl Manager {
    pub fn register<T: Default + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        options: StatePropertyOptions<T>,
        extract: Extract<ScalarValue>,
    ) -> RegisterResult<StateProperty<T>> {
        let handle = self.registry.register::<T>(ident, path, "", extract)?;
        handle.write(options.initial_value);

        let journal = JournalHandle::new(self.journal.clone());

        Ok(StateProperty {
            handle,
            journal,
            ident,
            name: path,
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
