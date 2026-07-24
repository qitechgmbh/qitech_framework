use super::PropertyHandle;

mod manager;
pub use manager::Manager;
pub use manager::MeasurementAccessHandle;
pub use manager::Reader;
pub use manager::Resolver;

#[derive(Debug)]
pub struct Measurement<T> {
    handle: PropertyHandle<T>,
}

impl<T: Copy> Measurement<T> {
    pub fn get(&self) -> T {
        *self.handle.read()
    }
}

impl<T> Measurement<T> {
    pub fn set(&mut self, value: T) {
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

/*
// --- statistics ---
#[derive(Debug)]
struct Statistics<T> {
    min: Option<PropertyHandle<T>>,
    max: Option<PropertyHandle<T>>,
}

impl<T: Copy> Statistics<T> {
    pub fn update(&mut self, value: T) {
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
