use std::marker::PhantomData;
use super::MachineBuilder;

impl<'a> MachineBuilder<'a> {
    pub fn state<'b, T, U>(
        &'b mut self, 
        name: &'static str
    ) -> StatePropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Default,
    {
        StatePropertyBuilder { 
            root: self, 
            name, 
            initial_value: Default::default(), 
            _marker: PhantomData 
        }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T, U = ()>
where
    T: Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> StatePropertyBuilder<'_, '_, T, U>
where
    T: Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> StateProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_state(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_state_handle(ident, name);

        StateProperty::new(data_handle, rec_handle, self.initial_value)
    }
}
