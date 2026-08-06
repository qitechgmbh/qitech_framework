use std::any::Any;
use std::mem::transmute;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::WriteCapability;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::build::BuildError;
use crate::machine::build::BuildResult;
use crate::resource::ConfigProperty;
use crate::resource::ConfigPropertyState;
use crate::resource::Erased;
use crate::resource::OnExternalChangedCallback;
use crate::resource::conversion::PropertyAdapter;
use crate::resource::conversion::PropertyType;

// --- config property ---
impl<'a> BuildContext<'a> {
    pub fn config<'b, T>(&'b mut self, path: &'static str) -> ConfigPropertyBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: PropertyAdapter + 'static,
    {
        ConfigPropertyBuilder {
            root: self,
            path,
            default: None,
            writable: WriteCapability::Allowed,
            constraints: <T::Type as PropertyType>::Constraints::default(),
            on_external_changed: None,
        }
    }
}

pub struct ConfigPropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,

    // --- configuration ---
    default: Option<T::Type>,
    writable: WriteCapability,
    constraints: <T::Type as PropertyType>::Constraints,
    on_external_changed: Option<OnExternalChangedCallback>,
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
    T::Type: Clone,
{
    pub fn default(mut self, value: T::Input) -> Self {
        self.default = Some(T::convert_input(value));
        self
    }

    pub fn allow_external_writes(mut self, value: bool) -> Self {
        self.writable = if value {
            WriteCapability::Forbidden {
                reason: "Initial state".to_string(),
            }
        } else {
            WriteCapability::Allowed
        };

        self
    }

    pub fn on_external_write<M: Machine + 'static>(mut self, func: fn(&mut M)) -> Self {
        fn adapter<M>(machine: &mut dyn Machine, func: *const ())
        where
            M: Machine + 'static,
        {
            let machine = (machine as &mut dyn Any)
                .downcast_mut::<M>()
                .expect("machine type mismatch");

            let func: fn(&mut M) = unsafe { transmute(func) };
            func(machine)
        }

        self.on_external_changed = Some(OnExternalChangedCallback {
            func: func as *const (),
            adapter: adapter::<M>,
        });

        self
    }

    pub fn register(self) -> BuildResult<ConfigProperty<T::Type>> {
        fn write<T: PropertyAdapter>(
            state: Erased,
            value_in: ScalarValue,
            value_out: Erased,
        ) -> Result<Option<ScalarValue>, ConfigPropertyWriteError>
        where
            T::Type: PartialEq,
        {
            let value = T::from_scalar(value_in)?;

            let state = state.downcast::<ConfigPropertyState<T::Type>>()
                .expect("Expected pointer to state");

            let state = unsafe { state.read() };

            if let WriteCapability::Forbidden { .. } = state.writable {
                return Err(ConfigPropertyWriteError::NotWritable);
            };

            if let Err(e) = (T::validate_constraints)(&state.constraints, &value) {
                return Err(ConfigPropertyWriteError::ConstraintViolation(e));
            }

            let mut value_out = value_out.downcast::<T::Type>()
                .expect("Expected pointer to value type");
            
            let value_out = unsafe { value_out.as_mut() };

            if value_out == &value {
                return Ok(None);
            }

            let before = T::into_scalar(value_out.clone());
            *value_out = value;
            Ok(Some(before))
        }

        let default = self
            .default
            .ok_or(BuildError::MissingRequiredField("default"))?;
        let writable = self.writable;

        // TODO: catch register error
        let handle = self.root.config_properties.register::<T::Type>(
            self.path,
            default.clone(),
            writable.clone(),
            self.constraints.clone(),
            write::<T>,
            self.on_external_changed,
        );

        // TODO: expose a temp journal so on failure we don't send this out
        self.root
            .journals
            .config_property
            .new_handle()
            .append(ConfigPropertyRecord {
                timestamp: Utc::now(),
                machine: self.root.ident,
                path: self.path.to_string(),
                event: ConfigPropertyEvent::Registered {
                    default: T::into_scalar(default.clone()),
                    capability: WriteCapability::Allowed,
                    constraints: Constraints::None,
                },
            });

        let prop = ConfigProperty::new(
            handle,
            T::into_scalar,
            T::validate_constraints,
            T::as_parameter_constraints,
            self.root.journals.config_property.new_handle(),
        );

        Ok(prop)
    }
}
