use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;
use std::rc::Weak;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::ConstraintViolationError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::with_uom_quantities;

use crate::journal::JournalHandle;
use crate::machine::ResourceKey;
use crate::machine::conversion::MutableConstraints;
use crate::machine::conversion::PropertyType;

pub struct ConfigPropertyState<T: PropertyType> {
    pub(crate) default: T,
    pub(crate) capability: WriteCapability,
    pub(crate) constraints: T::Constraints,
}

pub struct ConfigProperty<T: PropertyType> {
    pub(crate) state: Weak<RefCell<ConfigPropertyState<T>>>,
    pub(crate) key: ResourceKey,
    pub(crate) p_value: NonNull<T>,

    // --- conversion functions ---
    pub(crate) into_scalar: fn(T) -> ScalarValue,
    pub(crate) validate_constraints:
        fn(&T::Constraints, &T) -> Result<(), ConstraintViolationError>,
    pub(crate) as_parameter_constraints: fn(&T::Constraints) -> Constraints,

    // --- journals ---
    pub(crate) journal: JournalHandle<ConfigPropertyRecord>,
}

impl<T: PropertyType> ConfigProperty<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        if value == *self.get_ref() {
            self.record(ConfigPropertyEvent::Written {
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
                self.record(ConfigPropertyEvent::Written {
                    value: input,
                    origin: OperationOrigin::Machine,
                    outcome: ConfigPropertyWriteOutcome::Accepted { changed: true },
                });
            }

            Err(e) => {
                let err = ConfigPropertyWriteError::ConstraintViolation(e.clone());
                self.record(ConfigPropertyEvent::Written {
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
        let state = state.borrow();

        // --- abort if no change ---
        if state.default == value {
            return Ok(false);
        }

        // --- validate coonstraints ---
        (self.validate_constraints)(&state.constraints, &value)?;

        // --- record event ---
        let value = (self.into_scalar)(state.default.clone());
        self.record(ConfigPropertyEvent::DefaultChanged(value));
        Ok(true)
    }

    pub fn allow_external_write(&mut self) {
        self.set_writable(WriteCapability::Allowed);
    }

    pub fn forbid_external_write(&mut self, reason: impl Into<String>) {
        self.set_writable(WriteCapability::Forbidden {
            reason: reason.into(),
        });
    }

    pub fn set_constraints(
        &mut self,
        value: T::Constraints,
    ) -> Result<bool, ConstraintViolationError>
    where
        T::Constraints: MutableConstraints,
    {
        let state = self.state();
        let mut state = state.borrow_mut();

        // --- abort if no change ---
        if value == state.constraints {
            return Ok(false);
        }

        // ensure both current and default value are still valid with new constraints
        (self.validate_constraints)(&value, self.get_ref())?;
        (self.validate_constraints)(&value, &state.default)?;

        state.constraints = value;

        // --- record event ---
        let value = (self.as_parameter_constraints)(&state.constraints);
        drop(state);

        self.record(ConfigPropertyEvent::ConstraintsChanged(value));
        Ok(true)
    }
}

impl<T: PropertyType + Copy> ConfigProperty<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
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

        unsafe {
            (self.validate_constraints)(&state.constraints, &value)?;
            self.p_value.write(value);
        }

        Ok(())
    }

    fn set_writable(&mut self, value: WriteCapability) {
        let state = self.state();
        let state = state.borrow();

        // --- abort if no change ---
        if value == state.capability {
            return;
        }

        // --- record value ---
        let value = state.capability.clone();
        self.record(ConfigPropertyEvent::CapabilityChanged(value));
    }

    fn record(&mut self, event: ConfigPropertyEvent) {
        self.journal.append(ConfigPropertyRecord {
            timestamp: Utc::now(),
            machine: self.key.ident,
            path: self.key.path.to_string(),
            event,
        });
    }
}
