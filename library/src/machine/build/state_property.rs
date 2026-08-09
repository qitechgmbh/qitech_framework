use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::report::StatePropertyRecord;
use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::StateProperty;
use crate::resource::ResourceKey;
use crate::resource::conversion::PropertyAdapter;

impl<'a> BuildContext<'a> {
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

    // --- configuration ---
    value: T::Type,
}

impl<'a, 'b, T> StatePropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
    T::Type: Clone,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn register(self) -> Result<StateProperty<T::Type>, BuildError> {
        if !self.root.state_registered.insert(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let p_value = self.root.state.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            self.value.clone(),
            (),
        );

        self.root
            .journals_temp
            .state_property
            .new_handle()
            .append(StatePropertyRecord {
                timestamp: Utc::now(),
                machine: self.root.ident,
                path: self.path.to_string(),
                event: StatePropertyEvent::Registered {
                    value: T::into_scalar(self.value),
                },
            });

        let key = ResourceKey {
            ident: self.root.ident,
            path: self.path,
        };

        Ok(StateProperty {
            key,
            p_value,
            journal: self.root.journals.state_property.new_handle(),
            into_scalar: T::into_scalar,
        })
    }
}
