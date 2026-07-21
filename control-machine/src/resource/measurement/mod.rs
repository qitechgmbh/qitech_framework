use crate::conversion::{PropertyType, FloatPropertyType};
use crate::resource::{PropertyHandle, Specification, kind};

mod manager;
pub use manager::MeasurementManager;
pub use manager::MeasurementResolver;
pub use manager::MeasurementReader;
pub use manager::MeasurementAccessHandle;

pub trait MeasurementSpecification 
where 
    Self: Specification<Kind = kind::Measurement>,
    Self::Type: FloatPropertyType,
    <<Self as Specification>::Type as PropertyType>::Value: Copy
{
    // --- additional parameters ---
    const RECORD_MIN: bool = false;
    const RECORD_MAX: bool = false;

    // since uom ::new is not const we need this as func
    fn initial_value() -> <Self::Type as PropertyType>::Value;
}

#[derive(Debug)]
pub struct Measurement<T: PropertyType> {
    handle: PropertyHandle<T::Value>,
    // stats: Statistics<T>,
}

impl<T: PropertyType> Measurement<T> 
where 
    T::Value: Copy
{
    pub fn get(&self) -> T::Value {
        *self.handle.read()
    }
}

impl<T: PropertyType> Measurement<T> {
    pub fn set(&mut self, value: T::Value) {
        self.handle.write(value);
    }
}

/*
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl Measurement<$unit> {
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

        impl Measurement<Option<$unit>> {
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
*/

// --- statistics ---
// #[derive(Debug)]
// struct Statistics<T: PropertyType> {
//     min: Option< PropertyHandle<T::Value>>,
//     max: Option< PropertyHandle<T::Value>>,
// }

/*
impl<T: PropertyType> Statistics<T> {
    pub fn update(&mut self, value: T::Value) {
        let value = match value {
            Some(v) => v,
            None => return,
        };

        if let Some(min) = &mut self.min {
            match min.read() {
                Some(min) if value >= min => {}
                _ => min.write(Some(value)),
            }
        }

        if let Some(max) = &mut self.max {
            match max.read() {
                Some(max) if value <= max => {}
                _ => max.write(Some(value)),
            }
        }
    }
}
*/