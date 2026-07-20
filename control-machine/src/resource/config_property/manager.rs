use std::cell::{self, RefCell};
use std::rc::Rc;
use control_core::{MachineConfigMutation, MachineIdentificationUnique};

use crate::conversion::{Bounded, Wrapped};
use crate::resource::{ConstrainedConfigProperty, Journal, JournalHandle, PropertyRegistry, RegisterError};
use super::ConfigProperty;

pub struct ConfigPropertyManager {
    registry: PropertyRegistry<1, 512>,
    journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
}

impl ConfigPropertyManager {
    pub(crate) fn register<T>(
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

    pub(crate) fn register_constrained<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        default_value: T::Inner,
        initial_value: T::Inner,
        min: Option<<T::Inner as Bounded>::Bound>,
        max: Option<<T::Inner as Bounded>::Bound>,
        pred: Option<fn(&T::Inner) -> Result<(), String>>,
    ) -> Result<ConstrainedConfigProperty<T>, RegisterError> 
    where 
        T: Wrapped + 'static,
        T::Inner: Bounded
    {
        let handle = self.registry.register(ident, name)?;
        handle.write(initial_value);

        Ok(ConstrainedConfigProperty {
            handle,
            journal: JournalHandle::new(self.journal.clone()),
            ident,
            name,
            default: default_value,
            validate: todo!(),
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
