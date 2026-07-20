use std::marker::PhantomData;

use crate::{build::BuildError, conversion::Wrapped, resource::{StateProperty, StatePropertySpec}};
use super::BuildContext;

impl<'a> BuildContext<'a> {
    pub fn state<'b, T: StatePropertySpec>(&'b mut self) -> StatePropertyBuilder<'a, 'b, T>
    where 
        'a: 'b
    {
        StatePropertyBuilder { root: self, _marker: PhantomData }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T>
where
    T: StatePropertySpec
{
    root: &'b mut BuildContext<'a>,
    _marker: PhantomData<T>,
}

impl<T> StatePropertyBuilder<'_, '_, T>
where
    T: StatePropertySpec,
    <T::Value as Wrapped>::Inner: Default
{
    pub fn register(self) -> Result<StateProperty<T::Value>, BuildError> {
        let out = self.root.state_properties.register::<T::Value>(
            self.root.ident, 
            T::NAME, 
            T::initial_value()
        )?;

        Ok(out)
    }
}
