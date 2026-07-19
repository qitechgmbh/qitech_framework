use crate::{
    conversion::{
        Wrapped, 
        WrappedIntoOptionalF64,
    }, 
    data::MachineMeasurementWriteHandle, 
    with_uom,
};

#[derive(Debug)]
pub struct Measurement<T: Wrapped> {
    handle: MachineMeasurementWriteHandle,
    stats: MeasurementStatistics,
    value: T::Inner,
}

// scalar values
impl<T> Measurement<T>
where
    T: Wrapped,
    T:: Inner: Copy
{
    pub(super) fn new(
        handle: MachineMeasurementWriteHandle,
        stats: MeasurementStatistics,
        value: T::Inner,
    ) -> Self {
        Self {
            handle,
            stats,
            value,
        }
    }

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
        //self.stats.update(value);
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

// impl statistics
#[derive(Debug)]
pub struct MeasurementStatistics {
    min: Option<MachineMeasurementWriteHandle>,
    max: Option<MachineMeasurementWriteHandle>,
}

impl MeasurementStatistics {
    /*
    pub fn new(
        min: Option<MachineMeasurementWriteHandle>,
        max: Option<MachineMeasurementWriteHandle>,
    ) -> Self {
        Self { min, max }
    }

    pub fn update(&mut self, value: Option<f64>) {
        let value = match value {
            Some(v) => v,
            None => return,
        };

        if let Some(min) = &mut self.min {
            match min.get() {
                Some(min) if value >= min => {}
                _ => min.set(Some(value)),
            }
        }

        if let Some(max) = &mut self.max {
            match max.get() {
                Some(max) if value <= max => {}
                _ => max.set(Some(value)),
            }
        }
    }
    */
}
