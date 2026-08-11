use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::ConstraintViolationError;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::error::BuildError;

use crate::machine::ActResult;
use crate::machine::BuildContext;
use crate::machine::ConfigPropertyHandle;
use crate::machine::Machine;
use crate::machine::config_property::ConfigProperty;
use crate::machine::config_property::ConfigPropertyChangedCallbackFn;
use crate::machine::config_property::ConfigPropertyState;
use crate::resource::ResourceKey;
use crate::resource::constraints::NumericConstraints;
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
            constraints_error: None,
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
    constraints_error: Option<ConstraintViolationError>,

    on_external_write: Option<ConfigPropertyChangedCallbackFn>,
    on_external_write_error: Option<BuildError>,
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
    T::Type: PropertyType<Constraints = NumericConstraints<T::Type>> + PartialOrd + Copy,
{
    pub fn minimum(mut self, value: T::Input) -> Self {
        let value = T::convert_input(value);

        if let Some(max) = self.constraints.max
            && value > max
        {
            self.constraints_error = Some(ConstraintViolationError::IllegalRange {
                min: T::into_scalar(value),
                max: T::into_scalar(max),
            });

            return self;
        }

        self.constraints.min = Some(value);
        self
    }

    pub fn maximum(mut self, value: T::Input) -> Self {
        let value = T::convert_input(value);

        if let Some(min) = self.constraints.min
            && value < min
        {
            self.constraints_error = Some(ConstraintViolationError::IllegalRange {
                min: T::into_scalar(min),
                max: T::into_scalar(value),
            });

            return self;
        }

        self.constraints.max = Some(value);
        self
    }
}

impl<'a, 'b, const CAPACITY: usize> ConfigPropertyBuilder<'a, 'b, heapless::String<CAPACITY>> {
    pub fn length_min(mut self, value: usize) -> Self {
        if value > CAPACITY {
            self.constraints_error = Some(ConstraintViolationError::IllegalRange {
                min: ScalarValue::Integer(value as i64),
                max: ScalarValue::Integer(CAPACITY as i64),
            });

            return self;
        }

        self.constraints.min_length = Some(value);
        self
    }

    pub fn pattern(mut self, value: String) -> Self {
        let rgx = match regex::Regex::new(value.as_str()) {
            Ok(v) => v,
            Err(e) => {
                self.constraints_error = Some(ConstraintViolationError::IllegalPattern {
                    pattern: value,
                    error: e.to_string(),
                });
                return self;
            }
        };

        self.constraints.pattern = Some((value, rgx));
        self
    }
}

impl<'a, 'b, const CAPACITY: usize>
    ConfigPropertyBuilder<'a, 'b, Option<heapless::String<CAPACITY>>>
{
    pub fn min_length(mut self, value: usize) -> Self {
        if value > CAPACITY {
            self.constraints_error = Some(ConstraintViolationError::IllegalRange {
                min: ScalarValue::Integer(value as i64),
                max: ScalarValue::Integer(CAPACITY as i64),
            });

            return self;
        }

        self.constraints.min_length = Some(value);
        self
    }

    pub fn pattern(mut self, value: String) -> Self {
        let rgx = match regex::Regex::new(value.as_str()) {
            Ok(v) => v,
            Err(e) => {
                self.constraints_error = Some(ConstraintViolationError::IllegalPattern {
                    pattern: value,
                    error: e.to_string(),
                });
                return self;
            }
        };

        self.constraints.pattern = Some((value, rgx));
        self
    }
}

impl<'a, 'b, T> ConfigPropertyBuilder<'a, 'b, T>
where
    T: PropertyAdapter,
{
    pub fn default(mut self, value: T::Input) -> Self {
        self.default = Some(T::convert_input(value));
        self
    }

    pub fn forbid_external_writes(mut self) -> Self {
        self.capability = OperationCapability::Forbidden {
            reason: "Initial state".to_string(),
        };

        self
    }

    pub fn on_external_changed<M: Machine + 'static>(
        mut self,
        func: fn(&mut M) -> ActResult,
    ) -> Self {
        if self.root.type_id != TypeId::of::<M>() {
            self.on_external_write_error = Some(BuildError::IllegalMachineType {
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
        let Some(def) = self.root.schema.config_properties.get(self.path) else {
            return Err(BuildError::IllegalResourcePath {
                kind: ResourceKind::ConfigProperty,
                path: self.path.to_string(),
            });
        };

        if !T::validate_scalar_property_definition(def, false) {
            return Err(BuildError::IllegalResourceType {
                kind: ResourceKind::ConfigProperty,
                path: self.path.to_string(),
                expected: format!("{}", def.kind),
                received: type_name::<T>().to_string(),
            });
        }

        if self.root.config_registered.contains_key(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        if let Some(e) = self.constraints_error {
            return Err(BuildError::ConstraintViolation(e));
        }

        if let Some(err) = self.on_external_write_error {
            return Err(err);
        }

        let default = self.default.unwrap_or_default();

        // --- ensure initial value fits into constraints ---
        T::apply_constraints(&self.constraints, &default)?;

        let p_value = self.root.config.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            default.clone(),
            (),
        );

        let state = ConfigPropertyState {
            default: default.clone(),
            capability: self.capability.clone(),
            constraints: self.constraints.clone(),
        };

        let key = ResourceKey {
            ident: self.root.ident,
            path: self.path,
        };

        self.root
            .journals_temp
            .config_property
            .new_handle(key)
            .record(ConfigPropertyEvent::Registered {
                default: T::into_scalar(default.clone()),
                capability: self.capability.clone(),
                constraints: T::as_constraints(&self.constraints),
            });

        let state = Rc::new(RefCell::new(state));
        let state_for_write = Rc::clone(&state);

        let write = Box::new(move |value| {
            let value = T::from_scalar(value)?;

            let state = state_for_write.borrow();

            if let OperationCapability::Forbidden { .. } = state.capability {
                return Err(ConfigPropertyWriteError::NotWritable);
            };

            if let Err(e) = (T::apply_constraints)(&state.constraints, &value) {
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

        let journal = self.root.journals.config_property.new_handle(key);

        Ok(ConfigProperty {
            state: Rc::downgrade(&state),
            p_value,
            into_scalar: T::into_scalar,
            validate_constraints: T::apply_constraints,
            as_parameter_constraints: T::as_constraints,
            journal,
        })
    }
}
