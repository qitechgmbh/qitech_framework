use std::marker::PhantomData;

use crate::{
    ActResult, Machine, build::BuildError, resource::{ConfigProperty, ConfigPropertySpec, ConstrainedConfigProperty, ConstrainedConfigPropertySpec}
};

use super::BuildContext;

#[allow(type_alias_bounds)]
type OnApiChanged<M: Machine> = fn(&mut M) -> ActResult;

impl<'a> BuildContext<'a> {
    pub fn config<'b, T: ConfigPropertySpec>(&'b mut self) -> ConfigPropertyBuilder<'a, 'b, T>
    where 
        'a: 'b
    {
        ConfigPropertyBuilder { 
            root: self, 
            on_api_changed: None, 
            _marker: PhantomData,
        }
    }

    pub fn config_constrained<'b, T: ConstrainedConfigPropertySpec>(
        &'b mut self
    ) -> ConstrainedConfigPropertyBuilder<'a, 'b, T>
    where 
        'a: 'b
    {
        ConstrainedConfigPropertyBuilder { 
            root: self, 
            on_api_changed: None, 
            _marker: PhantomData,
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: ConfigPropertySpec
{
    root: &'b mut BuildContext<'a>,
    on_api_changed: Option<OnApiChanged<T::Value>>,
    _marker: PhantomData<T>,
}

impl<T> ConfigPropertyBuilder<'_, '_, T>
where
    T: ConfigPropertySpec,
{
    pub fn register(self) -> Result<ConfigProperty<T::Value>, BuildError> {
        _ = self.on_api_changed;

        let out = self.root.config_properties.register::<T::Value>(
            self.root.ident, 
            T::NAME, 
            T::default_value(),
            T::initial_value(),
        )?;

        Ok(out)
    }
}

pub struct ConstrainedConfigPropertyBuilder<'a, 'b, T>
where
    T: ConstrainedConfigPropertySpec
{
    root: &'b mut BuildContext<'a>,
    on_api_changed: Option<OnApiChanged<T::Value>>,
    _marker: PhantomData<T>,
}

impl<T> ConstrainedConfigPropertyBuilder<'_, '_, T>
where
    T: ConstrainedConfigPropertySpec,
{
    pub fn register(self) -> Result<ConstrainedConfigProperty<T::Value>, BuildError> {
        _ = self.on_api_changed;

        let out = self.root.config_properties.register::<T::Value>(
            self.root.ident, 
            T::NAME, 
            T::default_value(),
            T::initial_value(),
        )?;

        Ok(out)
    }
}
