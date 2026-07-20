use std::ptr::NonNull;
use crate::conversion::{Wrapped, WrappedIntoOptionalF64};

mod registry;
pub use registry::MeasurementManager;
pub use registry::MeasurementResolver;
pub use registry::MeasurementReader;

pub trait MeasurementSpec {
    // --- info ---
    const NAME: &'static str;
    type Value: 'static + Wrapped where <Self::Value as Wrapped>::Inner: Default;

    // --- additional parameters ---
    const RECORD_MIN: bool = false;
    const RECORD_MAX: bool = false;

    // since uom ::new is not const we need this cancer ...
    fn initial_value() -> <Self::Value as Wrapped>::Inner 
    where 
        <Self::Value as Wrapped>::Inner: Default;
}

#[derive(Debug)]
pub struct Measurement<T: Wrapped> {
    handle: Handle,
    value: T::Inner,
    stats: Statistics,
}

// scalar values
impl<T> Measurement<T>
where
    T: Wrapped,
    T:: Inner: Copy
{
    pub fn get(&self) -> T::Inner {
        self.value
    }
}

impl<T> Measurement<T> 
where
    T: WrappedIntoOptionalF64,
    T::Inner: Copy
{
    pub fn set(&mut self, value: T::Inner) {
        self.value = value;

        let value = T::into_opt_f64(value);
        self.handle.write(value);
        self.stats.update(value);
    }
}

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

#[derive(Debug, Clone)]
pub struct Config {
    record_min: bool,
    record_max: bool,
}

#[derive(Debug)]
struct Handle {
    p_value: NonNull<f64>,
    p_null: NonNull<bool>,
}   

impl Handle {
    fn read(&self) -> Option<f64> {
        unsafe {
            if self.p_null.read() {
                None
            } else {
                Some(self.p_value.read())
            }
        }
    }

    fn write(&mut self, value: Option<f64>) {
        unsafe {
            match value {
                Some(v) => {
                    self.p_value.write(v);
                    self.p_null.write(false);
                }
                None => {
                    self.p_null.write(true);
                }
            }
        }
    }
}

// --- statistics ---
#[derive(Debug)]
struct Statistics {
    min: Option<Handle>,
    max: Option<Handle>,
}

impl Statistics {
    pub fn update(&mut self, value: Option<f64>) {
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
