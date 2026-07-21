use std::borrow::Cow;

use chrono::Utc;
use control_core::{MachineConfigMutation, MachineIdentificationUnique, OperationResult, Origin};

use crate::resource::{JournalHandle, PropertyHandle};
use crate::conversion::{Bounded, PropertyType, ScalarPropertyType};

mod manager;
pub use manager::ConfigPropertyManager;
pub use manager::ConfigPropertyResolver;
pub use manager::ConfigPropertyReader;
pub use manager::ConfigPropertyAccessHandle;

pub struct ConfigPropertyOptions<T: Bounded> {
    default_value: T,
    min: T::Bound,
    max: T::Bound,
    pred: fn(&T) -> bool,
}

pub struct ConfigProperty<T: PropertyType> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T::Value>,
    journal: JournalHandle<MachineConfigMutation>,
    default: T::Value,
    pred: Option<fn(&T) -> bool>,
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
    T::Value: Clone,
{
    pub fn set(&mut self, value: T::Value) -> Result<(), String> {
        self.handle.write(value.clone());

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

impl<T> ConfigProperty<T>
where
    T: ScalarPropertyType,
    T::Value: Copy,
{
    pub fn get(&self) -> T::Value { *self.handle.read() }
}

// impl ConfigProperty<String> {
//     pub fn get(&self) -> &str { self.handle.read() }
// }
// 
// impl ConfigProperty<String> {
//     pub fn get(&self) -> &str { self.handle.read() }
// }

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
