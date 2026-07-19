use crate::{Machine, MachineBuildError, conversion::{Bounded, Wrapped}, machine::config::{ConfigProperty, FallibleConfigProperty}};
use super::MachineBuilder;

impl<'a> MachineBuilder<'a> {
    pub fn config<'b, T>(
        &'b mut self,
        name: &'static str,
        default_value: T::Inner,
    ) -> ConfigPropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Wrapped,
    {
        ConfigPropertyBuilder {
            root: self,
            name,
            default_value,
            initial_value: None,
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: Wrapped
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    default_value: T::Inner,
    initial_value: Option<T::Inner>,
}

impl<T> ConfigPropertyBuilder<'_, '_, T>
where
    T: Wrapped + 'static,
    T::Inner: Clone,
{
    pub fn initial_value(&mut self, value: T::Inner) -> &mut Self {
        self.initial_value = Some(value);
        self
    }

    pub fn register(self) -> Result<ConfigProperty<T>, MachineBuildError> {
        let ident = self.root.ident;

        let name = self.root.register_name(self.name);
        let mut reg_handle = self.root.data_store.registry.config.register(ident, name)?;

        if let Some(value) = self.initial_value {
            reg_handle.write(value);
        }

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        Ok(ConfigProperty::new(reg_handle, rec_handle, self.default_value))
    }
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: Wrapped + 'static,
    T::Inner: Clone + Bounded,
{
    pub fn with_lower_bound<M: Machine + 'static>(
        self, 
        bound: <T::Inner as Bounded>::Bound
    ) -> FallibleConfigPropertyBuilder<'a, 'b, T, M> {
        let mut builder = self.upgrade();
        builder.lower_bound = Some(bound);
        builder
    }

    pub fn with_upper_bound<M: Machine + 'static>(
        self, 
        bound: <T::Inner as Bounded>::Bound
    ) -> FallibleConfigPropertyBuilder<'a, 'b, T, M> {
        let mut builder = self.upgrade();
        builder.upper_bound = Some(bound);
        builder
    }

    pub fn with_validation<M: Machine + 'static>(
        self, 
        validate: fn(&M, &T::Inner) -> Result<(), String>,
    ) -> FallibleConfigPropertyBuilder<'a, 'b, T, M> {
        let mut builder = self.upgrade();
        builder.validate = Some(validate);
        builder
    }

    fn upgrade<M: Machine + 'static>(self) -> FallibleConfigPropertyBuilder<'a, 'b, T, M> {
        FallibleConfigPropertyBuilder {
            root: self.root,
            name: self.name,
            default_value: self.default_value,
            initial_value: self.initial_value,
            lower_bound: None,
            upper_bound: None,
            validate: None,
        }
    }
}

pub struct FallibleConfigPropertyBuilder<'a, 'b, T, M>
where
    T: Wrapped + 'static,
    T::Inner: Bounded,
    M: Machine,
{
    root: &'b mut MachineBuilder<'a>,
    name: &'static str,
    default_value: T::Inner,
    initial_value: Option<T::Inner>,

    lower_bound: Option<<T::Inner as Bounded>::Bound>,
    upper_bound: Option<<T::Inner as Bounded>::Bound>,
    validate: Option<fn(&M, &T::Inner) -> Result<(), String>>,
}

impl<M, T> FallibleConfigPropertyBuilder<'_, '_, T, M>
where
    T: Wrapped + 'static,
    T::Inner: Clone + Bounded,
    M: Machine + 'static,
{
    pub fn initial_value(mut self, value: T::Inner) -> Self {
        self.initial_value = Some(value);
        self
    }

    pub fn with_lower_bound(&mut self, bound: <T::Inner as Bounded>::Bound) -> &mut Self {
        self.lower_bound = Some(bound);
        self
    }

    pub fn with_upper_bound(&mut self, bound: <T::Inner as Bounded>::Bound) -> &mut Self {
        self.upper_bound = Some(bound);
        self
    }

    pub fn with_validation(&mut self, validate: fn(&M, &T::Inner) -> Result<(), String>) -> &mut Self {
        self.validate = Some(validate);
        self
    }

    pub fn register(self) -> Result<FallibleConfigProperty<M, T>, MachineBuildError> {
        let ident = self.root.ident;

        let name = self.root.register_name(self.name);
        let mut reg_handle = self.root.data_store.registry.config.register(ident, name)?;

        if let Some(value) = self.initial_value {
            reg_handle.write(value);
        }

        let rec = &mut self.root.data_store.recorder;
        let rec_handle = rec.create_config_handle(ident, name);

        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        let user_validate = self.validate;

        // Bounds are checked first; only if they pass does the user-supplied
        // validation function run.
        let validate: Box<dyn Fn(&M, &T::Inner) -> Result<(), String>> =
            Box::new(move |machine: &M, value: &T::Inner| {
                value
                    .validate(lower_bound, upper_bound)
                    .map_err(|e| e.to_string())?;

                if let Some(user_validate) = user_validate {
                    user_validate(machine, value)?;
                }

                Ok(())
            });

        Ok(FallibleConfigProperty::new(
            reg_handle, 
            rec_handle, 
            self.default_value,
            validate
        ))
    }
}
