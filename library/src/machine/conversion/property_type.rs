use std::fmt::Debug;

use qitech_framework_core::with_uom_quantities;

use crate::machine::constraints::NumericConstraints;
use crate::machine::constraints::OptionalNumericConstraints;
use crate::machine::constraints::OptionalStringConstraints;
use crate::machine::constraints::StringConstraints;
use crate::machine::constraints::Unconstrained;

pub trait MutableConstraints {}

pub trait PropertyType: Debug + Clone + PartialEq + Default + 'static {
    type Constraints: Debug + Clone + PartialEq + Default + 'static;
}

impl PropertyType for f64 {
    type Constraints = NumericConstraints<f64>;
}

impl PropertyType for Option<f64> {
    type Constraints = OptionalNumericConstraints<f64>;
}

impl PropertyType for i64 {
    type Constraints = NumericConstraints<i64>;
}

impl PropertyType for Option<i64> {
    type Constraints = OptionalNumericConstraints<i64>;
}

impl PropertyType for bool {
    type Constraints = Unconstrained;
}

impl PropertyType for Option<bool> {
    type Constraints = Unconstrained;
}

impl<const CAPACITY: usize> PropertyType for heapless::String<CAPACITY> {
    type Constraints = StringConstraints;
}

impl<const CAPACITY: usize> PropertyType for Option<heapless::String<CAPACITY>> {
    type Constraints = OptionalStringConstraints;
}

macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl PropertyType for $quantity {
            type Constraints = NumericConstraints<$quantity>;
        }

        impl PropertyType for Option<$quantity> {
            type Constraints = OptionalNumericConstraints<$quantity>;
        }
    };
}

with_uom_quantities!(impl_uom);
