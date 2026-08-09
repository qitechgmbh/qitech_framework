use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::ConstraintViolationError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::with_uom_quantities;

use crate::machine::ActResult;
use crate::machine::Machine;
use crate::resource::JournalHandle;
use crate::resource::constraints::EnumConstraints;
use crate::resource::constraints::NumericConstraints;
use crate::resource::conversion::PropertyType;

// --- functions ---
pub type ConfigPropertyWriteFn = Box<dyn Fn(ScalarValue) -> Result<bool, ConfigPropertyWriteError>>;
pub type ConfigPropertyChangedCallbackFn = Box<dyn Fn(&mut dyn Machine) -> ActResult>;

// --- property ---
pub struct ConfigProperty<T: PropertyType> {
    pub(crate) state: Weak<RefCell<ConfigPropertyState<T>>>,
    pub(crate) p_value: NonNull<T>,

    // --- conversion functions ---
    pub(crate) into_scalar: fn(T) -> ScalarValue,
    pub(crate) validate_constraints:
        fn(&T::Constraints, &T) -> Result<(), ConstraintViolationError>,
    pub(crate) as_parameter_constraints: fn(&T::Constraints) -> Constraints,

    // --- journal ---
    pub(crate) journal: JournalHandle<ConfigPropertyEvent>,
}

impl<T: PropertyType> ConfigProperty<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        if value == *self.get_ref() {
            self.journal.record(ConfigPropertyEvent::Written {
                value: (self.into_scalar)(value),
                origin: OperationOrigin::Machine,
                outcome: ConfigPropertyWriteOutcome::Accepted { changed: false },
            });

            return Ok(false);
        }

        let input = (self.into_scalar)(value.clone());
        let res = self.write(value.clone());

        match &res {
            Ok(_) => {
                self.journal.record(ConfigPropertyEvent::Written {
                    value: input,
                    origin: OperationOrigin::Machine,
                    outcome: ConfigPropertyWriteOutcome::Accepted { changed: true },
                });
            }

            Err(e) => {
                let err = ConfigPropertyWriteError::ConstraintViolation(e.clone());
                self.journal.record(ConfigPropertyEvent::Written {
                    value: input,
                    origin: OperationOrigin::Machine,
                    outcome: ConfigPropertyWriteOutcome::Rejected(err),
                });
            }
        }

        res.map(|_| true)
    }

    /// resets property back to the assigned default value
    pub fn reset(&mut self) -> Result<bool, ConstraintViolationError> {
        let state = self.state();
        let value = state.borrow().default.clone();
        self.set(value)
    }

    pub fn set_default(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        let state = self.state();
        let mut state = state.borrow_mut();

        // --- abort if no change ---
        if state.default == value {
            return Ok(false);
        }

        // --- validate constraints ---
        (self.validate_constraints)(&state.constraints, &value)?;

        // --- apply new value ---
        state.default = value.clone();

        // --- record event ---
        let value = (self.into_scalar)(state.default.clone());
        self.journal
            .record(ConfigPropertyEvent::DefaultChanged(value));

        Ok(true)
    }

    pub fn allow_external_write(&mut self) {
        self.set_writable(OperationCapability::Allowed);
    }

    pub fn forbid_external_write(&mut self, reason: impl Into<String>) {
        self.set_writable(OperationCapability::Forbidden {
            reason: reason.into(),
        });
    }
}

impl<T: PropertyType + Copy> ConfigProperty<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
    }
}

impl<T> ConfigProperty<T>
where
    T: Copy + PartialOrd + PropertyType<Constraints = NumericConstraints<T>>,
{
    pub fn set_min(&mut self, value: Option<T>) -> Result<bool, ConstraintViolationError> {
        let mut constraints = self.state().borrow().constraints.clone();
        constraints.set_min(value)?;

        self.set_constraints(constraints)
    }

    pub fn set_max(&mut self, value: Option<T>) -> Result<bool, ConstraintViolationError> {
        let mut constraints = self.state().borrow().constraints.clone();
        constraints.set_max(value)?;

        self.set_constraints(constraints)
    }

    pub fn set_min_clamped(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        let constraints = {
            let state = self.state();
            let mut state = state.borrow_mut();

            let mut constraints = state.constraints.clone();
            constraints.set_min(Some(value))?;

            // --- clamp current value ---
            unsafe {
                if self.p_value.read() < value {
                    self.p_value.write(value);
                }
            }

            // --- clamp default value ---
            if state.default < value {
                state.default = value;
            }

            constraints
        };

        self.set_constraints(constraints)
    }

    pub fn set_max_clamped(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        let constraints = {
            let state = self.state();
            let mut state = state.borrow_mut();

            let mut constraints = state.constraints.clone();
            constraints.set_max(Some(value))?;

            // --- clamp current value ---
            unsafe {
                if self.p_value.read() > value {
                    self.p_value.write(value);
                }
            }

            // --- clamp default value ---
            if state.default > value {
                state.default = value;
            }

            constraints
        };

        self.set_constraints(constraints)
    }
}

impl<T> ConfigProperty<T>
where
    T: PropertyType<Constraints = EnumConstraints<T>>,
{
    /// sets the list of allowed variants for this enum property
    pub fn set_allowed(&mut self, value: Vec<T>) -> Result<bool, ConstraintViolationError> {
        if value.is_empty() {
            return Err(ConstraintViolationError::NoAllowedVariants);
        }

        self.set_constraints(EnumConstraints { allowed: value })
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl ConfigProperty<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64) -> Result<bool, ConstraintViolationError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl ConfigProperty<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(
                &mut self,
                value: Option<f64>,
            ) -> Result<bool, ConstraintViolationError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);

// --- utils ---
impl<T: PropertyType> ConfigProperty<T> {
    fn state(&self) -> Rc<RefCell<ConfigPropertyState<T>>> {
        self.state.upgrade().expect("Outlived runtime instance")
    }

    fn write(&mut self, value: T) -> Result<(), ConstraintViolationError> {
        let state = self.state();
        let state = state.borrow();

        (self.validate_constraints)(&state.constraints, &value)?;

        unsafe {
            self.p_value.write(value);
        }

        Ok(())
    }

    fn set_writable(&mut self, value: OperationCapability) {
        let state = self.state();
        let mut state = state.borrow_mut();

        // --- abort if no change ---
        if value == state.capability {
            return;
        }

        // --- apply the change ---
        state.capability = value;

        // --- record value ---
        let value = state.capability.clone();
        self.journal
            .record(ConfigPropertyEvent::CapabilityChanged(value));
    }

    fn set_constraints(&mut self, value: T::Constraints) -> Result<bool, ConstraintViolationError> {
        let state = self.state();
        let mut state = state.borrow_mut();

        // --- abort if no change ---
        if value == state.constraints {
            return Ok(false);
        }

        // --- ensure existing values are still within constraints ---
        (self.validate_constraints)(&value, self.get_ref())?;
        (self.validate_constraints)(&value, &state.default)?;

        // --- apply the change ---
        state.constraints = value;

        // --- record event ---
        let value = (self.as_parameter_constraints)(&state.constraints);
        drop(state);

        self.journal
            .record(ConfigPropertyEvent::ConstraintsChanged(value));
        Ok(true)
    }
}

// --- state ---
pub struct ConfigPropertyState<T: PropertyType> {
    pub(crate) default: T,
    pub(crate) capability: OperationCapability,
    pub(crate) constraints: T::Constraints,
}

// --- handle ---
pub(crate) struct ConfigPropertyHandle {
    pub(crate) write: ConfigPropertyWriteFn,
    pub(crate) on_changed: Option<ConfigPropertyChangedCallbackFn>,
}
