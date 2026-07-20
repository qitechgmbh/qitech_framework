use std::cell::{self, RefCell};
use std::rc::Rc;
use control_core::{MachineIdentificationUnique, MachineStateMutation};

use crate::conversion::Wrapped;
use crate::resource::{Journal, JournalHandle, PropertyRegistry, RegisterError};
use super::StateProperty;

pub struct StatePropertyManager {
    registry: PropertyRegistry<0, 512>,
    journal: Rc<RefCell<Journal<MachineStateMutation>>>,
}

impl StatePropertyManager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        initial_value: T::Inner,
    ) -> Result<StateProperty<T>, RegisterError> 
    where 
        T: Wrapped + 'static,
    {
        let handle = self.registry.register(ident, name)?;
        handle.write(initial_value);

        Ok(StateProperty {
            handle,
            journal: JournalHandle::new(self.journal.clone()),
            ident,
            name,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub fn journal(&self) -> cell::Ref<'_, Journal<MachineStateMutation>> {
        self.journal.borrow()
    }

    pub fn clear_journal(&self) {
        self.journal.borrow_mut().clear();
    }
}
