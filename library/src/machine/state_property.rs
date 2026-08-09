use std::ptr::NonNull;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::with_uom_quantities;

use crate::resource::JournalHandle;
use crate::resource::ResourceKey;
use crate::resource::conversion::PropertyType;

pub struct StateProperty<T: PropertyType> {
    pub(crate) key: ResourceKey,
    pub(crate) p_value: NonNull<T>,
    pub(crate) into_scalar: fn(T) -> ScalarValue,
    pub(crate) journal: JournalHandle<StatePropertyEvent>,
}

impl<T: PropertyType> StateProperty<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> bool {
        if value == *self.get_ref() {
            return false;
        }

        unsafe {
            // --- write the value ---
            self.p_value.write(value.clone());
        }

        // --- record the change ---
        let value = (self.into_scalar)(value);
        self.journal
            .record(StatePropertyEvent::ValueChanged { value });

        true
    }
}

impl<T: PropertyType + Copy> StateProperty<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl StateProperty<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64) -> bool
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl StateProperty<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>) -> bool
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);
