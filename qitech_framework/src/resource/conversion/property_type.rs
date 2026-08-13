use std::fmt::Debug;

use qitech_framework_core::with_uom_quantities;

use crate::resource::constraints::NumericConstraints;
use crate::resource::constraints::StringConstraints;
use crate::resource::constraints::Unconstrained;

pub trait PropertyType: Debug + Clone + PartialEq + Default + 'static {
    type Constraints: Debug + Clone + PartialEq + Default + 'static;
}

impl PropertyType for f64 {
    type Constraints = NumericConstraints<f64>;
}

impl PropertyType for i64 {
    type Constraints = NumericConstraints<i64>;
}

impl PropertyType for bool {
    type Constraints = Unconstrained;
}

impl<const CAPACITY: usize> PropertyType for heapless::String<CAPACITY> {
    type Constraints = StringConstraints;
}

macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl PropertyType for $quantity {
            type Constraints = NumericConstraints<$quantity>;
        }
    };
}

with_uom_quantities!(impl_uom);

impl<T> PropertyType for Option<T>
where
    T: PropertyType,
{
    type Constraints = T::Constraints;
}
