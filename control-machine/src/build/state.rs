use crate::{build::MachineBuildError, conversion::Wrapped, resource::StateProperty};
use super::MachineBuildContext;

impl<'a> MachineBuildContext<'a> {
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
    root: &'b mut MachineBuildContext<'a>,
    name: &'static str,
    initial_value: Option<T::Inner>
}

impl<T> StatePropertyBuilder<'_, '_, T>
where
    T: Wrapped + 'static,
    T::Inner: Default
{
    pub fn initial_value(&mut self, value: T::Inner) -> &mut Self {
        self.initial_value = Some(value);
        self
    }

    pub fn register(self) -> Result<StateProperty<T>, MachineBuildError> {
        let out = self.root.state_properties.register::<T>(
            self.root.ident, 
            self.name, 
            self.initial_value.unwrap_or_default()
        )?;

        Ok(out)
    }
}
