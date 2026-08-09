use std::cell::Cell;
use std::ptr::NonNull;
use std::rc::Rc;

use qitech_framework_core::with_uom_quantities;

use crate::conversion::PropertyType;
use crate::conversion::StatisticValue;

pub struct Measurement<T>
where
    T: PropertyType + StatisticValue,
{
    pub(crate) p_value: NonNull<T>,
    pub(crate) stats: MeasurementStatistics<T>,
}

impl<T> Measurement<T>
where
    T: PropertyType + StatisticValue,
{
    pub fn get_ref(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> bool {
        let changed = unsafe { value != *self.p_value.as_ref() };

        unsafe {
            // always update stats
            self.stats.update(value);
        }

        if changed {
            unsafe {
                self.p_value.write(value);
            }
        }

        changed
    }
}

impl<T: PropertyType + Copy> Measurement<T>
where
    T: PropertyType + StatisticValue,
{
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

// --- statistics ---
#[derive(Debug)]
pub(crate) struct MeasurementStatistics<T: StatisticValue> {
    /// cycle generation, used to know when to reset stats
    pub(crate) p_generation: Rc<Cell<u64>>,
    pub(crate) generation: u64,

    pub(crate) min: Option<NonNull<T>>,
    pub(crate) max: Option<NonNull<T>>,
    pub(crate) avg: Option<NonNull<T>>,
    pub(crate) stddev: Option<NonNull<T>>,

    pub(crate) count: u64,
    pub(crate) mean: f64,
    pub(crate) m2: f64,
}

impl<T: StatisticValue> MeasurementStatistics<T> {
    pub unsafe fn update(&mut self, value: T) {
        let generation_now = self.p_generation.get();
        let is_new_generation = generation_now != self.generation;

        if is_new_generation {
            self.generation = generation_now;
            self.count = 0;
            self.mean = 0.0;
            self.m2 = 0.0;
        }

        if let Some(min) = &mut self.min {
            unsafe {
                if is_new_generation || value < min.read() {
                    min.write(value);
                }
            }
        }

        if let Some(max) = &mut self.max {
            unsafe {
                if is_new_generation || value > max.read() {
                    max.write(value);
                }
            }
        }

        if let Some(value_f64) = value.as_opt_f64()
            && (self.avg.is_some() || self.stddev.is_some())
        {
            self.count += 1;

            let delta = value_f64 - self.mean;
            self.mean += delta / self.count as f64;

            let delta2 = value_f64 - self.mean;
            self.m2 += delta * delta2;

            if let Some(avg) = &mut self.avg {
                unsafe {
                    avg.write(T::from_f64(self.mean));
                }
            }

            if let Some(stddev) = &mut self.stddev {
                let variance = if self.count > 1 {
                    self.m2 / (self.count - 1) as f64
                } else {
                    0.0
                };

                unsafe {
                    stddev.write(T::from_f64(variance.sqrt()));
                }
            }
        }
    }
}
