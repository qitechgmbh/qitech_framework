use crate::{MachineBuildError, conversion::Wrapped, data, machine::StateProperty};
use super::MachineBuilder;

impl<'a> MachineBuilder<'a> {
    pub fn state<'b, T>(
        &'b mut self, 
        name: &'static str
    ) -> StatePropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Wrapped,
    {
        StatePropertyBuilder { 
            root: self, 
            name, 
            initial_value: Default::default(), 
        }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T>
where
    T: Wrapped
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    initial_value: Option<T::Inner>
}

impl<T> StatePropertyBuilder<'_, '_, T>
where
    T: Wrapped + 'static,
{
    pub fn initial_value(&mut self, value: T::Inner) -> &mut Self {
        self.initial_value = Some(value);
        self
    }

    pub fn register(self) -> Result<StateProperty<T>, MachineBuildError> {
        let ident = self.root.ident;

        let name = self.root.register_name(self.name);
        let mut reg_handle = self.root.data_store.registry.state.register(ident, name)?;

        if let Some(value) = self.initial_value {
            reg_handle.write(value);
        }

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_state_handle(ident, name);

        Ok(StateProperty::new(reg_handle, rec_handle))
    }
}
