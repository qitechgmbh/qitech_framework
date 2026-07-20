use std::borrow::Cow;

use chrono::Utc;
use qitech_lib::units::*;
use control_core::{MachineConfigMutation, MachineIdentificationUnique, OperationResult, Origin};

use crate::with_uom;
use crate::resource::{JournalHandle, PropertyHandle};
use crate::conversion::{Wrapped, WrappedIntoScalar};

mod manager;
pub use manager::ConfigPropertyManager;

pub struct ConfigProperty<T: Wrapped> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T::Inner>,
    journal: JournalHandle<MachineConfigMutation>,
    default: T::Inner,
}

impl<T> ConfigProperty<T> 
where 
    T: Wrapped,
    T::Inner: Clone
{
    /// reset property back to default value
    pub fn reset(&mut self) {
        self.handle.write(self.default.clone());
    }
}

impl<T> ConfigProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Copy,
{
    pub fn get(&self) -> T::Inner { *self.handle.read() }

    pub fn set(&mut self, value: T::Inner) {
        self.handle.write(value);

        self.journal.append(MachineConfigMutation { 
            timestamp: Utc::now(), 
            ident: self.ident, 
            name: Cow::Borrowed(self.name), 
            value: T::into_scalar(value),
            origin: Origin::Machine,
            result: OperationResult::Success,
        });
    }
}

// fallible variant
#[allow(type_alias_bounds)]
pub type ConstrainedConfigPropertyValidateFn<T: Wrapped> = Box<dyn Fn(&T::Inner) -> Result<(), String>>;

pub struct ConstrainedConfigProperty<T: Wrapped> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T::Inner>,
    journal: JournalHandle<MachineConfigMutation>,
    default: T::Inner,
    validate: ConstrainedConfigPropertyValidateFn<T>,
}

impl<T> ConstrainedConfigProperty<T> 
where 
    T: Wrapped,
    T::Inner: Clone
{
    /// reset property to default value
    pub fn reset(&mut self) {
        self.handle .write(self.default.clone());
    }
}

impl<T> ConstrainedConfigProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Copy,
{
    pub fn get(&self) -> T::Inner { *self.handle.read() }

    pub fn set(&mut self, value: T::Inner) -> Result<(), String> {
        let mut journal_entry = MachineConfigMutation { 
            timestamp: Utc::now(), 
            ident: self.ident, 
            name: Cow::Borrowed(self.name), 
            value: T::into_scalar(value),
            origin: Origin::Machine,
            result: OperationResult::Success,
        };

        if let Err(e) = (self.validate)(&value) {
            journal_entry.result = OperationResult::Failure;
            self.journal.append(journal_entry);
            return Err(e);
        }

        self.handle.write(value);
        self.journal.append(journal_entry);
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