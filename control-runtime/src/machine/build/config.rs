use std::marker::PhantomData;

use crate::{Machine, MachineBuildError, conversion::{Bounded, Wrapped, in_bounds}, machine::config::{ConfigProperty, ConstrainedConfigProperty}};
use super::MachineBuildContext;

impl<'a> MachineBuildContext<'a> {
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
    root: &'b mut MachineBuildContext<'a>,
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
        let mut reg_handle = self.root.resource_registry
            .register_config_property(ident, name)?;

        if let Some(value) = self.initial_value {
            reg_handle.write(value);
        }

        let rec = &mut self.root.resource_journals;
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
        validate: fn(&T::Inner) -> Result<(), String>,
    ) -> FallibleConfigPropertyBuilder<'a, 'b, T, M> {
        let mut builder = self.upgrade();
        builder.validate = Some(validate);
        builder
    }

    pub fn with_on_changed() {

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
            validate_api_change: None,
            after_api_change: None,
            _marker: PhantomData,
        }
    }
}

#[allow(type_alias_bounds)]
type ValidateFn<T: Wrapped> = fn(&T::Inner) -> Result<(), String>;

type ValidateApiChangeFn<T: Wrapped> = fn(&T::Inner) -> Result<(), String>;

type AfterApiChanged<M: Machine> = fn(&mut M) -> Result<(), ()>;

pub struct FallibleConfigPropertyBuilder<'a, 'b, T, M>
where
    T: Wrapped + 'static,
    T::Inner: Bounded,
    M: Machine + 'static
{
    root: &'b mut MachineBuildContext<'a>,
    name: &'static str,
    default_value: T::Inner,
    initial_value: Option<T::Inner>,

    lower_bound: Option<<T::Inner as Bounded>::Bound>,
    upper_bound: Option<<T::Inner as Bounded>::Bound>,

    validate: Option<ValidateFn<T>>,
    validate_api_change: Option<ValidateApiChangeFn<T>>,
    after_api_change: Option<AfterApiChanged<T>>,
    _marker: PhantomData<M>
}

impl<T, M> FallibleConfigPropertyBuilder<'_, '_, T, M>
where
    T: Wrapped + 'static,
    T::Inner: Clone + Bounded,
    M: Machine + 'static
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

    pub fn with_validation(&mut self, validate: fn(&T::Inner) -> Result<(), String>) -> &mut Self {
        self.validate = Some(validate);
        self
    }

    pub fn with_before_api_changed(mut self) -> Self {
        self
    }

    pub fn with_after_api_changed(mut self) -> Self {
        self
    }

    pub fn register(self) -> Result<ConstrainedConfigProperty<T>, MachineBuildError> {
        let ident = self.root.ident;

        let name = self.root.register_name(self.name);
        let mut reg_handle = self.root.data_store.registry.config.register(ident, name)?;

        if let Some(value) = self.initial_value {
            reg_handle.write(value);
        }

        let rec = &mut self.root.data_store.journals;
        let rec_handle = rec.create_config_handle(ident, name);

        let lower_bound = self.lower_bound;
        let upper_bound = self.upper_bound;
        let user_validate = self.validate;

        // Bounds are checked first; only if they pass does the user-supplied
        // validation function run.
        let validate: Box<dyn Fn(&T::Inner) -> Result<(), String>> =
            Box::new(move |value: &T::Inner| {
                if !in_bounds(value, lower_bound, upper_bound) {
                    return Err("out of bounds".to_string());
                }

                if let Some(user_validate) = user_validate {
                    user_validate(value)?;
                }

                Ok(())
            });

        Ok(ConstrainedConfigProperty::new(
            reg_handle, 
            rec_handle, 
            self.default_value,
            validate
        ))
    }
}
