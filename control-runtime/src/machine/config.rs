use std::{fmt, marker::PhantomData};
use control_core::{OperationResult, Origin, ScalarValue};
use qitech_lib::units::*;

use crate::{data::{ConfigRecorderHandle, PropertyHandle}, machine::to_scalar::ToScalar};

pub struct ConfigProperty<T, U = ()> {
    // handles
    data_handle: PropertyHandle,
    rec_handle: ConfigRecorderHandle,

    // data
    default: T,
    value: T,
    _unit: PhantomData<U>,
}

impl<T: Clone, U> ConfigProperty<T, U> {
    pub(crate) fn new(
        data_handle: PropertyHandle,
        rec_handle: ConfigRecorderHandle,
        default: T,
        value: T,
    ) -> Self {
        Self {
            data_handle,
            rec_handle,
            default,
            value,
            _unit: PhantomData,
        }
    }

    /// reset property to default value
    pub fn reset(&mut self) {
        self.value = self.default.clone()
    }
}

impl<T: Clone + ToScalar> ConfigProperty<T> {
    pub fn set(&mut self, value: T, origin: Origin) {
        self.value = value.clone();
        let value = value.to_scalar();
        self.data_handle.write(value.clone());
        self.rec_handle
            .record_mutation(origin, value, OperationResult::Success);
    }
}

// bounded variant
pub struct BoundedConfigProperty<T: Bounded, U = ()> {
    sink: ConfigRecorderHandle,
    prop: PropertyHandle,
    bounds: Bounds<T>,
    default: T,
    value: T,
    _unit: PhantomData<U>,
}

impl<T: Clone + Bounded, U> BoundedConfigProperty<T, U> {
    pub(crate) fn new(
        prop: PropertyHandle,
        sink: ConfigRecorderHandle,
        bounds: Bounds<T>,
        default: T,
        value: T,
    ) -> Self {
        assert!(bounds.contains(&value), "Default must be in bounds");

        Self {
            prop,
            sink,
            bounds,
            default,
            value,
            _unit: PhantomData,
        }
    }

    /// reset property to default value
    pub fn reset(&mut self) {
        self.value = self.default.clone()
    }
}

impl<T: Bounded + Clone + ToScalar> BoundedConfigProperty<T> {
    pub fn set(&mut self, value: T, origin: Origin) -> Result<(), BoundsError<T>> {
        self.bounds.check(&value)?;
        self.value = value.clone();
        let value = value.to_scalar();
        self.prop.write(value.clone());
        self.sink
            .record_mutation(origin, value, OperationResult::Success);
        Ok(())
    }
}

// uom impl
macro_rules! impl_uom {
    ($quantity:ty, $unit:tt) => {
        impl<U> ConfigProperty<$quantity, U>
        where
            U: $unit::Unit + $unit::Conversion<f64>,
        {
            pub fn get(&self) -> $quantity {
                self.value
            }

            pub fn get_native(&self) -> f64 {
                self.value.get::<U>()
            }

            pub fn get_as<N>(&self) -> f64
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.value.get::<N>()
            }

            pub fn set(&mut self, value: $quantity, origin: Origin) {
                self.value = value.clone();
                let value: ScalarValue = value.get::<U>().to_scalar();
                self.data_handle.write(value.clone());
                self.rec_handle
                    .record_mutation(origin, value, OperationResult::Success);
            }

            pub fn set_as<N>(&mut self, value: f64, origin: Origin)
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.set(<$quantity>::new::<N>(value), origin);
            }
        }

        impl<U> BoundedConfigProperty<$quantity, U>
        where
            U: $unit::Unit + $unit::Conversion<f64>,
        {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.value.get::<N>()
            }

            pub fn set(&mut self, value: $quantity, origin: Origin) -> Result<(), BoundsError<$quantity>> {
                self.bounds.check(&value)?;
                self.value = value.clone();
                let value: ScalarValue = value.get::<U>().to_scalar();
                self.prop.write(value.clone());
                self.sink.record_mutation(origin, value, OperationResult::Success);
                Ok(())
            }

            pub fn set_as<N>(&mut self, value: f64, origin: Origin) -> Result<(), BoundsError<$quantity>>
            where
                N: $unit::Unit + $unit::Conversion<f64>,
            {
                self.set(<$quantity>::new::<N>(value), origin)
            }
        }

        impl Bounded for $quantity {
            type Bound = $quantity;
            fn as_bound(&self) -> $quantity {
                *self
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

// Bounding
pub trait Bounded {
    type Bound: PartialOrd + Copy + Default;
    fn as_bound(&self) -> Self::Bound;
}

// Numeric / uom-style types: bound directly against the value itself.
impl Bounded for f64 {
    type Bound = f64;
    fn as_bound(&self) -> f64 {
        *self
    }
}

// String: bound against length, not lexicographic value.
impl Bounded for String {
    type Bound = usize;
    fn as_bound(&self) -> usize {
        self.len() // or .chars().count() if you want char count, not bytes
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Bounds<T: Bounded> {
    pub min: Option<T::Bound>,
    pub max: Option<T::Bound>,
}

impl<T: Bounded> Bounds<T> {
    pub fn contains(&self, value: &T) -> bool {
        let b = value.as_bound();
        match (self.min, self.max) {
            (Some(min), Some(max)) => b >= min && b <= max,
            (Some(min), None) => b >= min,
            (None, Some(max)) => b <= max,
            (None, None) => true,
        }
    }

    pub fn check(&self, value: &T) -> Result<(), BoundsError<T>> {
        if !self.contains(value) {
            return Err(BoundsError {
                value: value.as_bound(),
                min: self.min,
                max: self.max,
            });
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BoundsError<T: Bounded> {
    pub value: T::Bound,
    pub min: Option<T::Bound>,
    pub max: Option<T::Bound>,
}

impl<T> fmt::Display for BoundsError<T>
where
    T: Bounded,
    T::Bound: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => write!(
                f,
                "value {} is outside the bounds [{}, {}]",
                self.value, min, max
            ),
            (Some(min), None) => write!(
                f,
                "value {} is below the minimum of {}",
                self.value, min
            ),
            (None, Some(max)) => write!(
                f,
                "value {} is above the maximum of {}",
                self.value, max
            ),
            (None, None) => unreachable!("Can't be out of bounds if no bounds defined ..."),
        }
    }
}

impl<T> std::error::Error for BoundsError<T>
where
    T: Bounded + fmt::Debug,
    T::Bound: fmt::Debug + fmt::Display,
{}

// --- attachment handle ---
