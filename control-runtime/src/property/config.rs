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

