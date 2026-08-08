use qitech_framework_core::with_uom_units;

pub trait Extract<T> {
    /// Extracts a value from a raw byte pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` is valid for reads of the required
    /// size and properly aligned for the expected type.
    unsafe fn extract(bytes: *const ()) -> T;
}

// --- float ---
impl Extract<Option<f64>> for f64 {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        let value = unsafe { *(bytes as *const f64) };
        Some(value)
    }
}

impl Extract<Option<f64>> for Option<f64> {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        unsafe { *(bytes as *const Option<f64>) }
    }
}

// --- integer ---
impl Extract<Option<f64>> for i64 {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        let value = unsafe { *(bytes as *const i64) };
        Some(value as f64)
    }
}

impl Extract<Option<f64>> for Option<i64> {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        let value = unsafe { *(bytes as *const Option<i64>) };
        value.map(|v| v as f64)
    }
}

// --- boolean ---
impl Extract<Option<f64>> for bool {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        let value = unsafe { *(bytes as *const bool) };
        Some(if value { 1.0 } else { 0.0 })
    }
}

impl Extract<Option<f64>> for Option<bool> {
    unsafe fn extract(bytes: *const ()) -> Option<f64> {
        let value = unsafe { *(bytes as *const Option<bool>) };
        value.map(|v| if v { 1.0 } else { 0.0 })
    }
}

// --- uom ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl Extract<Option<f64>> for $unit {
            unsafe fn extract(bytes: *const ()) -> Option<f64> {
                let value = unsafe { *(bytes as *const $quantity) };
                Some(value.get::<$unit>())
            }
        }

        impl Extract<Option<f64>> for Option<$unit> {
            unsafe fn extract(bytes: *const ()) -> Option<f64> {
                let value = unsafe { *(bytes as *const Option<$quantity>) };
                value.map(|v| v.get::<$unit>())
            }
        }
    };
}

with_uom_units!(impl_uom_unit);
