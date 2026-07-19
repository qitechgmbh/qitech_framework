use std::marker::PhantomData;
use super::MachineBuilder;

impl<'a> MachineBuilder<'a> {
    pub fn config<'b, T, U>(
        &'b mut self,
        name: &'static str,
        default_value: T,
    ) -> ConfigPropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Copy + Default,
    {
        ConfigPropertyBuilder {
            root: self,
            name,
            default_value,
            initial_value: default_value,
            _marker: PhantomData,
        }
    }

    pub fn config_bounded<'b, T, U>(
        &'b mut self,
        name: &'static str,
        default_value: T,
    ) -> BoundedConfigPropertyBuilder<'a, 'b, T, U>
    where
        'a: 'b,
        T: Bounded + Copy + Default,
    {
        BoundedConfigPropertyBuilder {
            root: self,
            name,
            bounds: Default::default(),
            default_value,
            initial_value: default_value,
            _marker: PhantomData,
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T, U = ()>
where
    T: Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    default_value: T,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> ConfigPropertyBuilder<'_, '_, T, U>
where
    T: Clone + Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> ConfigProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_config(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        ConfigProperty::new(data_handle, rec_handle, self.default_value, self.initial_value)
    }
}

pub struct BoundedConfigPropertyBuilder<'a, 'b, T, U = ()>
where
    T: Bounded + Default,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    bounds: Bounds<T>,
    default_value: T,
    initial_value: T,
    _marker: PhantomData<U>,
}

impl<T: Default, U> BoundedConfigPropertyBuilder<'_, '_, T, U>
where
    T: Bounded + Clone + Default,
{
    pub fn initial_value(&mut self, value: T) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn min(&mut self, value: T) -> &mut Self {
        self.bounds.min = Some(value.as_bound());
        self
    }

    pub fn max(&mut self, value: T) -> &mut Self {
        self.bounds.max = Some(value.as_bound());
        self
    }

    pub fn register(self) -> BoundedConfigProperty<T, U> {
        let ident = self.root.ident;

        let reg = &mut self.root.data_store.registry;
        let name = reg.register_name(self.name.to_string());
        let data_handle = reg.register_config(ident, name).unwrap();

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        BoundedConfigProperty::new(
            data_handle, 
            rec_handle, 
            self.bounds,
            self.default_value, 
            self.initial_value
        )
    }
}