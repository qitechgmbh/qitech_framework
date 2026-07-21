use std::marker::PhantomData;

use crate::{build::BuildError, conversion::{PropertyType, ScalarPropertyType}, resource::{Specification, StateProperty, StatePropertySpecification}};
use super::BuildContext;

impl<'a> BuildContext<'a> {
    pub fn state<'b, Spec: StatePropertySpecification>(&'b mut self) -> StatePropertyBuilder<'a, 'b, Spec>
    where 
        'a: 'b,
        <Spec as Specification>::Type: ScalarPropertyType,
    {
        StatePropertyBuilder { root: self, _marker: PhantomData }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T>
where
    T: StatePropertySpecification,
    <T as Specification>::Type: ScalarPropertyType
{
    root: &'b mut BuildContext<'a>,
    _marker: PhantomData<T>,
}

impl<Spec> StatePropertyBuilder<'_, '_, Spec>
where
    Spec: StatePropertySpecification + 'static,
    <Spec as Specification>::Type: ScalarPropertyType,
{
    pub fn register(
        self, 
        initial_value: <Spec::Type as PropertyType>::Value,
    ) -> Result<StateProperty<Spec::Type>, BuildError> {
        let out = self.root.state_properties.register::<Spec>(
            self.root.ident, 
            initial_value,
        )?;

        Ok(out)
    }
}
