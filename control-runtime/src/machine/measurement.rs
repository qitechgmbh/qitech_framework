use qitech_lib::units::*;

use crate::{conversion::FloatRepr, data::MachineMeasurementWriteHandle};

#[derive(Debug)]
pub struct Measurement<T: FloatRepr> {
    handle: MachineMeasurementWriteHandle,
    value: T::Value,
    stats: MeasurementStatistics,
}

// scalar values
impl<V: FloatRepr> Measurement<V> {
    pub(super) fn new(
        handle: MachineMeasurementWriteHandle,
        stats: MeasurementStatistics,
        initial_value: V::Value,
    ) -> Self {
        Self { handle, value: initial_value, stats }
    }

    pub fn get(&self) -> V::Value { self.value }

    pub fn set(&mut self, value: V::Value) {
        self.value = value;

        let value = V::to_f64(self.value);
        self.handle.write(Some(V::to_f64(self.value)));
        self.stats.update(Some(value));
    }
}

// scalar variants

impl Measurement<Option<i64>> {
    pub fn set(&mut self, value: Option<i64>) {
        self.value = value;
        let value = value.map(|v| v as f64);
        self.handle.set(value);
        self.stats.update(value);
    }
}

impl Measurement<Option<bool>> {
    pub fn set(&mut self, value: Option<bool>) {
        self.value = value;
        let value = value.map(f64::from);
        self.handle.set(value);
        self.stats.update(value);
    }
}

// nullable variants
macro_rules! impl_uom {
    ($quantity:ty, $unit:tt) => {
        impl<U> Measurement<$quantity, U>
        where
            U: $unit::Unit + $unit::Conversion<f64>,
        {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.value.get::<N>()
            }

            pub fn get_native(&self) -> f64 {
                self.value.get::<U>()
            }

            pub fn set(&mut self, value: $quantity) {
                self.value = value;
                let value = Some(value.get::<U>());
                self.handle.set(value);
                self.stats.update(value);
            }

            pub fn set_as<N>(&mut self, value: f64)
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.set(<$quantity>::new::<N>(value));
            }
        }

        impl<U> Measurement<Option<$quantity>, U>
        where
            U: $unit::Unit + $unit::Conversion<f64>,
        {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                if let Some(v) = self.value {
                    return Some(v.get::<N>());
                }

                None
            }

            pub fn get_native(&self) -> Option<f64> {
                self.value.map(|x| x.get::<U>())
            }

            pub fn set(&mut self, value: Option<$quantity>) {
                self.value = value;
                let value = value.map(|x| x.get::<U>());
                self.handle.set(value);
                self.stats.update(value);
            }

            pub fn set_as<N>(&mut self, value: Option<f64>)
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                let value = value.map(|x| <$quantity>::new::<N>(x));
                self.set(value);
            }
        }
    };
}

impl_uom!(Acceleration, acceleration);
impl_uom!(AmountOfSubstance, amount_of_substance);
impl_uom!(Angle, angle);
impl_uom!(AngularAcceleration, angular_acceleration);
impl_uom!(AngularJerk, angular_jerk);
impl_uom!(AngularVelocity, angular_velocity);
impl_uom!(ElectricCurrent, electric_current);
impl_uom!(ElectricPotential, electric_potential);
impl_uom!(Frequency, frequency);
impl_uom!(Jerk, jerk);
impl_uom!(Length, length);
impl_uom!(LuminousIntensity, luminous_intensity);
impl_uom!(Mass, mass);
impl_uom!(Pressure, pressure);
impl_uom!(Ratio, ratio);
impl_uom!(ThermodynamicTemperature, thermodynamic_temperature);
impl_uom!(Time, time);
impl_uom!(Velocity, velocity);
impl_uom!(VolumeRate, volume_rate);

// impl statistics
#[derive(Debug)]
pub struct MeasurementStatistics {
    min: Option<MeasurementHandle>,
    max: Option<MeasurementHandle>,
}

impl MeasurementStatistics {
    pub fn new(
        min: Option<MeasurementHandle>,
        max: Option<MeasurementHandle>,
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
}
