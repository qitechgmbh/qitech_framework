use std::marker::PhantomData;

use crate::{build::BuildError, conversion::{Bounded, PropertyType, ScalarPropertyType}, resource::{ConfigProperty, ConfigPropertySpecification, Specification}};
use super::BuildContext;

impl<'a> BuildContext<'a> {
    pub fn config<'b, T>(&'b mut self) -> ConfigPropertyBuilder<'a, 'b, T>
    where 
        'a: 'b,
        T: ConfigPropertySpecification,
        <T as Specification>::Type: ScalarPropertyType,
        <<T as Specification>::Type as PropertyType>::Value: Bounded
    {
        ConfigPropertyBuilder { root: self, _marker: PhantomData }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: ConfigPropertySpecification,
    <T as Specification>::Type: ScalarPropertyType,
    <<T as Specification>::Type as PropertyType>::Value: Bounded
{
    root: &'b mut BuildContext<'a>,
    _marker: PhantomData<T>,
}

impl<Spec> ConfigPropertyBuilder<'_, '_, Spec>
where
    Spec: ConfigPropertySpecification + 'static,
    <Spec as Specification>::Type: ScalarPropertyType,
    <<Spec as Specification>::Type as PropertyType>::Value: Bounded
{
    pub fn register(self) -> Result<ConfigProperty<Spec::Type>, BuildError> {
        let out = self.root.config_properties.register::<Spec>(self.root.ident)?;
        Ok(out)
    }
}
