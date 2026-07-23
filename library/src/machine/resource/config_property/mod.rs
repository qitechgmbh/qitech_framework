use std::borrow::Cow;
use chrono::Utc;

use qitech_framework_common::{
    MachineIdentificationUnique, 
    MachineConfigMutation, 
    OperationResult, 
    OperationOrigin
};

use super::{
    JournalHandle, 
    PropertyHandle,
    BoundedMeta,
};

mod manager;
pub use manager::Manager;
pub use manager::Resolver;
pub use manager::Reader;
pub use manager::AccessHandle;

pub struct ConfigPropertyOptions<T: BoundedMeta> {
    default_value: T,
    min: T::Bound,
    max: T::Bound,
    pred: fn(&T) -> bool,
}

pub struct ConfigProperty<T: Clone> {
    ident: MachineIdentificationUnique,
    resource_path: &'static str,
    handle: PropertyHandle<T>,
    journal: JournalHandle<MachineConfigMutation>,
    default: T,
    pred: Option<fn(&T) -> bool>,
}

impl<T: Clone> ConfigProperty<T> {
    /// reset property back to default value
    pub fn reset(&mut self) {
        self.handle.write(self.default.clone());
    }
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), String> {
        self.handle.write(value.clone());

        self.journal.append(MachineConfigMutation { 
            target: self.ident, 
            resource_path: Cow::Borrowed(self.resource_path), 
            value: T::into_scalar(value),
            origin: OperationOrigin::Machine,
            result: OperationResult::Success,
            timestamp: Utc::now(), 
        });

        Ok(())
    }
}

impl<T: Clone + Copy> ConfigProperty<T> {
    pub fn get(&self) -> T { *self.handle.read() }
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
