use std::ptr::NonNull;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::report::StatePropertyRecord;
use qitech_framework_core::with_uom_quantities;

use crate::journal::JournalHandle;
use crate::machine::ResourceKey;
use crate::machine::conversion::PropertyType;

pub struct StateProperty<T: PropertyType> {
    pub(crate) key: ResourceKey,
    pub(crate) p_value: NonNull<T>,
    pub(crate) journal: JournalHandle<StatePropertyRecord>,
    pub(crate) into_scalar: fn(T) -> ScalarValue,
}

impl<T: PropertyType> StateProperty<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> bool {
        if value == *self.get_ref() {
            self.record(StatePropertyEvent::ValueChanged {
                value: (self.into_scalar)(value),
            });

            return false;
        }

        unsafe {
            self.p_value.write(value);
        }

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

// --- utils ---
impl<T: PropertyType> StateProperty<T> {
    fn record(&mut self, event: StatePropertyEvent) {
        self.journal.append(StatePropertyRecord {
            timestamp: Utc::now(),
            machine: self.key.ident,
            path: self.key.path.to_string(),
            event,
        });
    }
}
