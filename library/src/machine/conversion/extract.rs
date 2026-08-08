use std::ptr::NonNull;

use qitech_framework_core::with_uom_units;

pub trait Extract<T> {
    /// Extracts a value from a non-null pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` points to a valid, properly
    /// aligned value of the expected type for the duration of the call.
    unsafe fn extract(bytes: NonNull<()>) -> T;
}

// --- float ---
impl Extract<Option<f64>> for f64 {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        let value = unsafe { bytes.cast::<f64>().as_ref() };
        Some(*value)
    }
}

impl Extract<Option<f64>> for Option<f64> {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        *unsafe { bytes.cast::<Option<f64>>().as_ref() }
    }
}

// --- integer ---
impl Extract<Option<f64>> for i64 {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        let value = unsafe { *bytes.cast::<i64>().as_ref() };
        Some(value as f64)
    }
}

impl Extract<Option<f64>> for Option<i64> {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        let value = unsafe { bytes.cast::<Option<i64>>().as_ref() };
        value.map(|v| v as f64)
    }
}

// --- boolean ---
impl Extract<Option<f64>> for bool {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        let value = unsafe { *bytes.cast::<bool>().as_ref() };
        Some(if value { 1.0 } else { 0.0 })
    }
}

impl Extract<Option<f64>> for Option<bool> {
    unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
        let value = unsafe { bytes.cast::<Option<bool>>().as_ref() };
        value.map(|v| if v { 1.0 } else { 0.0 })
    }
}

// --- uom ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl Extract<Option<f64>> for $unit {
            unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
                let value = unsafe { bytes.cast::<$quantity>().as_ref() };
                Some(value.get::<$unit>())
            }
        }

        impl Extract<Option<f64>> for Option<$unit> {
            unsafe fn extract(bytes: NonNull<()>) -> Option<f64> {
                let value = unsafe { bytes.cast::<Option<$quantity>>().as_ref() };
                value.map(|v| v.get::<$unit>())
            }
        }
    };
}

with_uom_units!(impl_uom_unit);
