use std::borrow::Cow;

use chrono::Utc;
use control_core::{MachineConfigMutation, MachineIdentificationUnique, OperationResult, Origin};

use crate::resource::{JournalHandle, PropertyHandle, kind};
use crate::conversion::{Bounded, PropertyType, ScalarPropertyType};

mod manager;
pub use manager::ConfigPropertyManager;
pub use manager::ConfigPropertyResolver;
pub use manager::ConfigPropertyReader;
pub use manager::ConfigPropertyAccessHandle;

pub trait ConfigPropertySpecification 
where
    Self: super::Specification<Kind = kind::ConfigProperty>,
    Self::Type: ScalarPropertyType,
    <Self::Type as PropertyType>::Value: Bounded,
{
    // since uom ::new is not const we need a func ...
    fn default_value() -> <Self::Type as PropertyType>::Value;

    const MIN: Option<<<Self::Type as PropertyType>::Value as Bounded>::Bound> = None;
    const MAX: Option<<<Self::Type as PropertyType>::Value as Bounded>::Bound> = None;

    fn validate(value: &<Self::Type as PropertyType>::Value) -> Result<(), String> {
        _ = value;
        Ok(())
    }
}

pub struct ConfigProperty<T: PropertyType> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T::Value>,
    journal: JournalHandle<MachineConfigMutation>,
    default: T::Value,
    // validate: fn(&T::Value) -> Result<(), String>,
}

impl<T> ConfigProperty<T> 
where 
    T: PropertyType,
    T::Value: Clone
{
    /// reset property back to default value
    pub fn reset(&mut self) {
        self.handle.write(self.default.clone());
    }
}

impl<T> ConfigProperty<T>
where
    T: ScalarPropertyType,
    T::Value: Copy,
{
    pub fn get(&self) -> T::Value { *self.handle.read() }

    pub fn set(&mut self, value: T::Value) -> Result<(), String> {
        self.handle.write(value);

        self.journal.append(MachineConfigMutation { 
            timestamp: Utc::now(), 
            ident: self.ident, 
            name: Cow::Borrowed(self.name), 
            value: T::into_scalar(value),
            origin: Origin::Machine,
            result: OperationResult::Success,
        });

        Ok(())
    }
}

/*
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
*/
