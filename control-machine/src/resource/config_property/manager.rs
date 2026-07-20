use std::cell::{self, RefCell};
use std::rc::Rc;
use control_core::{MachineConfigMutation, MachineIdentificationUnique};

use crate::conversion::Wrapped;
use crate::resource::{Journal, JournalHandle, PropertyRegistry, RegisterError};
use super::ConfigProperty;

pub struct ConfigPropertyManager {
    registry: PropertyRegistry<1, 512>,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl ConfigPropertyManager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        default_value: T::Inner,
        initial_value: T::Inner,
    ) -> Result<ConfigProperty<T>, RegisterError> 
    where 
        T: Wrapped + 'static,
    {
        let handle = self.registry.register(ident, name)?;
        handle.write(initial_value);

        Ok(ConfigProperty {
            handle,
            journal: JournalHandle::new(self.journal.clone()),
            ident,
            name,
            default: default_value,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }

    pub fn journal(&self) -> cell::Ref<'_, Journal<MachineConfigMutation>> {
        self.journal.borrow()
    }

    pub fn clear_journal(&self) {
        self.journal.borrow_mut().clear();
    }
}
