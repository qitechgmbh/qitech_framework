use std::ptr::NonNull;

use qitech_framework_core::with_uom_units;

pub trait ReadMeasurement {
    /// Loads and converts a value from a raw, non-null pointer into 
    /// the measurements export format Option<f64>;
    /// 
    /// # Safety
    /// 
    /// `ptr` must point to a valid instance of `T`.
    unsafe fn read(ptr: NonNull<()>) -> Option<f64>;
}

// --- float ---
impl ReadMeasurement for f64 {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        let value = unsafe { ptr.cast::<f64>().as_ref() };
        Some(*value)
    }
}

impl ReadMeasurement for Option<f64> {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        *unsafe { ptr.cast::<Option<f64>>().as_ref() }
    }
}

// --- integer ---
impl ReadMeasurement for i64 {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        let value = unsafe { *ptr.cast::<i64>().as_ref() };
        Some(value as f64)
    }
}

impl ReadMeasurement for Option<i64> {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        let value = unsafe { ptr.cast::<Option<i64>>().as_ref() };
        value.map(|v| v as f64)
    }
}

// --- boolean ---
impl ReadMeasurement for bool {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        let value = unsafe { *ptr.cast::<bool>().as_ref() };
        Some(if value { 1.0 } else { 0.0 })
    }
}

impl ReadMeasurement for Option<bool> {
    unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
        let value = unsafe { ptr.cast::<Option<bool>>().as_ref() };
        value.map(|v| if v { 1.0 } else { 0.0 })
    }
}

// --- uom ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl ReadMeasurement for $unit {
            unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
                let value = unsafe { ptr.cast::<$quantity>().as_ref() };
                Some(value.get::<$unit>())
            }
        }

        impl ReadMeasurement for Option<$unit> {
            unsafe fn read(ptr: NonNull<()>) -> Option<f64> {
                let value = unsafe { ptr.cast::<Option<$quantity>>().as_ref() };
                value.map(|v| v.get::<$unit>())
            }
        }
    };
}

with_uom_units!(impl_uom_unit);
