use qitech_lib::units::*;
use control_core::{OperationResult, Origin};

use crate::with_uom;
use crate::data::{property, ConfigRecorderHandle};
use crate::conversion::{Wrapped, WrappedIntoScalar};

pub struct ConfigProperty<T: Wrapped> {
    reg_handle: property::Handle<T::Inner>,
    rec_handle: ConfigRecorderHandle,
    default: T::Inner,
}

impl<T> ConfigProperty<T> 
where 
    T: Wrapped,
    T::Inner: Clone
{
    pub(crate) fn new(
        reg_handle: property::Handle<T::Inner>,
        rec_handle: ConfigRecorderHandle,
        default: T::Inner,
    ) -> Self {
        Self { reg_handle, rec_handle, default }
    }

    /// reset property to default value
    pub fn reset(&mut self) {
        self.reg_handle .write(self.default.clone());
    }
}

impl<T> ConfigProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Copy,
{
    pub fn get(&self) -> T::Inner { *self.reg_handle.read() }

    pub fn set(&mut self, value: T::Inner) {
        self.reg_handle.write(value);
        self.rec_handle.record(Origin::Machine, T::into_scalar(value), OperationResult::Success);
    }
}

// fallible variant
pub struct ConstrainedConfigProperty<T: Wrapped> {
    reg_handle: property::Handle<T::Inner>,
    rec_handle: ConfigRecorderHandle,
    default: T::Inner,
    validate: Box<dyn Fn(&T::Inner) -> Result<(), String>>,
}

impl<T> ConstrainedConfigProperty<T> 
where 
    T: Wrapped,
    T::Inner: Clone
{
    pub(crate) fn new(
        reg_handle: property::Handle<T::Inner>,
        rec_handle: ConfigRecorderHandle,
        default: T::Inner,
        validate: Box<dyn Fn(&T::Inner) -> Result<(), String>>,
    ) -> Self {
        Self { reg_handle, rec_handle, default, validate }
    }

    /// reset property to default value
    pub fn reset(&mut self) {
        self.reg_handle .write(self.default.clone());
    }
}

impl<T> ConstrainedConfigProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Copy,
{
    pub fn get(&self) -> T::Inner { *self.reg_handle.read() }

    pub fn set(&mut self, value: T::Inner) -> Result<(), String> {
        if let Err(e) = (self.validate)(&value) {
            self.rec_handle.record(Origin::Machine, T::into_scalar(value), OperationResult::Failure);
            return Err(e);
        }

        self.reg_handle.write(value);
        self.rec_handle.record(Origin::Machine, T::into_scalar(value), OperationResult::Success);
        Ok(())
    }
}

// uom impl
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl ConfigProperty<$unit> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value));
            }
        }

        impl ConfigProperty<Option<$unit>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>));
            }
        }

        impl ConstrainedConfigProperty<$unit> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64) -> Result<(), String>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl ConstrainedConfigProperty<Option<$unit>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>) -> Result<(), String>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom!(impl_uom);