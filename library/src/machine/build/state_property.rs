use std::any::type_name;
use std::borrow::Cow;

use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::StateProperty;
use crate::resource::ResourceKey;
use crate::resource::conversion::PropertyAdapter;

impl<'a> BuildContext<'a> {
    /// Creates a builder for a state property.
    ///
    /// The property is registered when [`StatePropertyBuilder::build`] is called.
    pub fn state<'b, T>(&'b mut self, path: &'static str) -> StatePropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: PropertyAdapter + 'static,
    {
        StatePropertyBuilder {
            root: self,
            path,
            value: T::Type::default(),
        }
    }
}

pub struct StatePropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,
    value: T::Type,
}

impl<'a, 'b, T> StatePropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
    T::Type: Clone,
{
    /// Sets the initial value of the state property.
    ///
    /// If omitted, the property's [`Default`] value is used.
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn build(self) -> Result<StateProperty<T::Type>, BuildError> {
        let Some(def) = self.root.schema.state_properties.get(self.path) else {
            return Err(BuildError::IllegalResourcePath {
                kind: ResourceKind::StateProperty,
                path: self.path.to_string(),
            });
        };

        if !T::validate_scalar_property_definition(def) {
            return Err(BuildError::IllegalResourceType {
                kind: ResourceKind::StateProperty,
                path: self.path.to_string(),
                expected: format!("{}", def.kind),
                received: type_name::<T>().to_string(),
            });
        }

        if !self.root.state_registered.insert(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let p_value = self.root.state.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            self.value.clone(),
            (),
        );

        let key = ResourceKey {
            ident: self.root.ident,
            path: self.path,
        };

        self.root
            .journals_temp
            .state_property
            .new_handle(key)
            .record(StatePropertyEvent::Registered {
                value: T::into_scalar(self.value),
            });

        let journal = self.root.journals.state_property.new_handle(key);

        Ok(StateProperty {
            p_value,
            journal,
            into_scalar: T::into_scalar,
        })
    }
}
