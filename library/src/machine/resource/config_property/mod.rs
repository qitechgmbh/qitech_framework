use qitech_framework_common::{
    MachineIdentificationUnique, 
    MachineConfigMutation, 
};

use crate::machine::error::BoundsError;

use super::{JournalHandle, PropertyHandle};

mod manager;
pub use manager::Manager;
pub use manager::Resolver;
pub use manager::Reader;
pub use manager::AccessHandle;

pub enum ApiWriteConfigError {
    JournalFull,
    ParseError(serde_json::Error),
    ValueOutOfBounds(BoundsError),
    ValidateError(String),
}

pub enum WriteConfigError {
    JournalFull,
    ValueOutOfBounds(BoundsError),
    ValidateError(String),
}

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    journal: JournalHandle<MachineConfigMutation>,
    validate: manager::ValidateAndRecord<T>,
    default: T,

}

impl<T: Clone> ConfigProperty<T> {
    /// reset property back to default value
    pub fn reset(&mut self) {
        self.handle.write(self.default.clone());
    }
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), WriteConfigError> {
        (self.validate)(&mut self.journal, &value)?;
        self.handle.write(value);
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
