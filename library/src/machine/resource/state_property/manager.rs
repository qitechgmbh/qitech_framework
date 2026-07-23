use std::cell::{Ref, RefCell};
use std::rc::Rc;
use qitech_framework_common::{MachineIdentificationUnique, MachineStateMutation, ScalarValue};

use crate::machine::resource::property::ExtractFn;
use crate::machine::resource::{
    Journal, 
    JournalHandle, 
    PropertyReadHandle, 
    PropertyReader,
    PropertyRegistry, 
    PropertyResolver, 
    error::RegisterResult,
    kind,
};

use super::StateProperty;

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type Resolver<'a> = PropertyResolver<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type Reader<'a> = PropertyReader<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    ScalarValue
>;

pub type ReadHandle<T> = PropertyReadHandle<kind::StateProperty, T>;

pub struct Manager {
    registry: Registry,
    journal: Rc<RefCell<Journal<MachineStateMutation>>>,
}

impl Manager {
    pub fn register<T: Default + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        initial_value: T,
        extract: ExtractFn<ScalarValue>,
    ) -> RegisterResult<StateProperty<T>> {
        let handle = self.registry.register::<T>(ident, path, "", extract, ())?;
        handle.write(initial_value);

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
