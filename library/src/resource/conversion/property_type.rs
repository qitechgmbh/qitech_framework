use qitech_framework_core::with_uom_quantities;

use crate::resource::constraints::NumericConstraints;
use crate::resource::constraints::OptionalNumericConstraints;
use crate::resource::constraints::OptionalStringConstraints;
use crate::resource::constraints::StringConstraints;

pub trait PropertyType: Clone + PartialEq + Default + 'static {
    type Constraints: Clone + PartialEq + Default + 'static;
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
    type Constraints = ();
}

impl PropertyType for Option<bool> {
    type Constraints = ();
}

impl PropertyType for String {
    type Constraints = StringConstraints;
}

impl PropertyType for Option<String> {
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
