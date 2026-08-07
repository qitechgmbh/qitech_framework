use chrono::Utc;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::report::StatePropertyRecord;
use qitech_framework_core::report::error::BuildResult;

use crate::machine::BuildContext;
use crate::resource::StateProperty;
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

    pub fn register(self) -> BuildResult<StateProperty<T::Type>> {
        // TODO: catch register error
        let handle = self
            .root
            .state_properties
            .register::<T::Type>(self.path, self.value.clone());

        // TODO: expose a temp journal so on failure we don't send this out
        self.root
            .journals
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

        let prop = StateProperty::new(
            handle,
            T::into_scalar,
            self.root.journals.state_property.new_handle(),
        );

        Ok(prop)
    }
}
