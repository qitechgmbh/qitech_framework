use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use chrono::Utc;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::ConfigPropertyChangedCallbackFn;
use crate::machine::Machine;
use crate::machine::config_property::ConfigProperty;
use crate::machine::config_property::ConfigPropertyState;
use crate::machine::error::ActResult;
use crate::machine::instance::ConfigPropertyHandle;
use crate::resource::ResourceKey;
use crate::resource::conversion::PropertyAdapter;
use crate::resource::conversion::PropertyType;

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
            capability: OperationCapability::Allowed,
            constraints: <T::Type as PropertyType>::Constraints::default(),
            on_external_write_error: None,
            on_external_write: None,
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
    capability: OperationCapability,
    constraints: <T::Type as PropertyType>::Constraints,

    on_external_write_error: Option<BuildError>,
    on_external_write: Option<ConfigPropertyChangedCallbackFn>,
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter,
{
    pub fn default(mut self, value: T::Input) -> Self {
        self.default = Some(T::convert_input(value));
        self
    }

    pub fn allow_external_writes(mut self, value: bool) -> Self {
        self.capability = if value {
            OperationCapability::Forbidden {
                reason: "Initial state".to_string(),
            }
        } else {
            OperationCapability::Allowed
        };

        self
    }

    pub fn on_external_changed<M: Machine + 'static>(
        mut self,
        func: fn(&mut M) -> ActResult,
    ) -> Self {
        if self.root.type_id != TypeId::of::<M>() {
            // returning an error in the build call is not ergonomic thus we store
            // the error and invoke it on register();
            self.on_external_write_error = Some(BuildError::MachineTypeMismatch {
                expected: self.root.type_name.to_string(),
                received: type_name::<M>().to_string(),
            });

            return self;
        }

        self.on_external_write = Some(Box::new(move |machine: &mut dyn Machine| {
            let machine = (machine as &mut dyn Any)
                .downcast_mut::<M>()
                .expect("machine type mismatch");

            func(machine)
        }));

        self
    }

    pub fn build(self) -> Result<ConfigProperty<T::Type>, BuildError> {
        if self.root.config_registered.contains_key(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        if let Some(err) = self.on_external_write_error {
            return Err(err);
        }

        let default = self.default.unwrap_or_default();

        let p_value = self.root.config.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            default.clone(),
            (),
        );

        let state = ConfigPropertyState {
            default: default.clone(),
            capability: self.capability,
            constraints: self.constraints,
        };

        self.root
            .journals_temp
            .config_property
            .new_handle()
            .append(ConfigPropertyRecord {
                timestamp: Utc::now(),
                machine: self.root.ident,
                path: self.path.to_string(),
                event: ConfigPropertyEvent::Registered {
                    default: T::into_scalar(default.clone()),
                    capability: OperationCapability::Allowed,
                    constraints: Constraints::None,
                },
            });

        let key = ResourceKey {
            ident: self.root.ident,
            path: self.path,
        };

        let state = Rc::new(RefCell::new(state));
        let state_for_write = Rc::clone(&state);

        let write = Box::new(move |value| {
            let value = T::from_scalar(value)?;

            let state = state_for_write.borrow();

            if let OperationCapability::Forbidden { .. } = state.capability {
                return Err(ConfigPropertyWriteError::NotWritable);
            };

            if let Err(e) = (T::validate_constraints)(&state.constraints, &value) {
                return Err(ConfigPropertyWriteError::ConstraintViolation(e));
            }

            unsafe {
                if p_value.read() == value {
                    return Ok(false);
                }

                p_value.write(value);
            }

            Ok(true)
        });

        self.root.config_registered.insert(
            self.path,
            ConfigPropertyHandle {
                write,
                on_changed: self.on_external_write,
            },
        );

        Ok(ConfigProperty {
            state: Rc::downgrade(&state),
            key,
            p_value,
            into_scalar: T::into_scalar,
            validate_constraints: T::validate_constraints,
            as_parameter_constraints: T::as_parameter_constraints,
            journal: self.root.journals.config_property.new_handle(),
        })
    }
}
