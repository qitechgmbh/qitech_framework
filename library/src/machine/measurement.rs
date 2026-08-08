use std::ptr::NonNull;

use qitech_framework_core::with_uom_quantities;

use crate::machine::conversion::PropertyType;

pub struct Measurement<T: PropertyType> {
    pub(crate) p_value: NonNull<T>,
}

impl<T: PropertyType> Measurement<T> {
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> bool {
        if value == *self.get_ref() {
            return false;
        }

        unsafe {
            self.p_value.write(value);
        }

        true
    }
}

impl<T: PropertyType + Copy> Measurement<T> {
    pub fn get(&self) -> T {
        *self.get_ref()
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl Measurement<$quantity> {
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

        impl Measurement<Option<$quantity>> {
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
