use std::borrow::Cow;

use chrono::Utc;
use control_core::MachineIdentificationUnique;
use control_core::MachineStateMutation;

use crate::with_uom;
use crate::resource::PropertyHandle;
use crate::resource::JournalHandle;
use crate::conversion::{Wrapped, WrappedIntoScalar};

mod manager;

#[derive(Debug)]
pub struct StateProperty<T: Wrapped> {
    ident: MachineIdentificationUnique,
    name: &'static str,
    handle: PropertyHandle<T::Inner>,
    journal: JournalHandle<MachineStateMutation>,
}

impl<T> StateProperty<T>
where
    T: WrappedIntoScalar,
    T::Inner: Clone,
{
    pub fn get(&self) -> &T::Inner { self.handle.read() }

    pub fn set(&mut self, value: T::Inner) {
        self.handle.write(value.clone());
        self.journal.append(MachineStateMutation { 
            timestamp: Utc::now(), 
            ident: self.ident, 
            name: Cow::Borrowed(self.name), 
            value: T::into_scalar(value),
        });
    }
}

macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl StateProperty<$unit> {
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

        impl StateProperty<Option<$unit>> {
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
    };
}

with_uom!(impl_uom);