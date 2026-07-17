use std::marker::PhantomData;
use control_core::ScalarValue;
use qitech_lib::units::*;
use crate::{data::{PropertyHandle, StateRecorderHandle}, machine::to_scalar::ToScalar};

#[derive(Debug)]
pub struct StateProperty<T, U = ()> {
    data_handle: PropertyHandle,
    rec_handle: StateRecorderHandle,
    value: T,
    _unit: PhantomData<U>,
}

impl<T, U> StateProperty<T, U> {
    pub fn new(
        data_handle: PropertyHandle,
        rec_handle: StateRecorderHandle,
        value: T,
    ) -> Self {
        Self { data_handle, rec_handle, value, _unit: PhantomData }
    }
}

impl<T> StateProperty<T, ()>
where
    T: ToScalar + Clone,
{
    pub fn get(&self) -> &T { &self.value }

    pub fn set(&mut self, value: T) {
        self.value = value.clone();

        let value = value.clone().to_scalar();
        self.data_handle.write(value.clone());
        self.rec_handle.record_mutation(value);
    }
}

macro_rules! impl_uom {
    ($quantity:ty, $unit:tt) => {
        impl<U> StateProperty<$quantity, U>
        where
            U: $unit::Unit + $unit::Conversion<f64>,
        {
            pub fn get(&self) -> $quantity {
                self.value
            }

            pub fn get_as<N>(&self) -> f64
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.value.get::<N>()
            }

            pub fn set(&mut self, value: $quantity) {
                self.value = value;

                let value: ScalarValue = self.value.get::<U>().to_scalar();
                self.data_handle.write(value.clone());
                self.rec_handle.record_mutation(value);
            }

            pub fn set_as<N>(&mut self, value: f64)
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.set(<$quantity>::new::<N>(value));
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