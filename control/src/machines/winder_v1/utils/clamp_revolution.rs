use qitech_framework::uom::angle::revolution;
use qitech_framework::uom::f64::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clamping {
    None,
    Min,
    Max,
}

// Clamps a UOM angle value to be within the specified range [min, max].
// This is a wrapper around [`clamp_revolution`] that works with UOM Angle types.
pub fn clamp_revolution_uom(value: Angle, min: Angle, max: Angle) -> (Angle, Clamping) {
    let value = value.get::<revolution>();
    let min = min.get::<revolution>();
    let max = max.get::<revolution>();

    // clamp
    let clamped_value = clamp_revolution(value, min, max);

    // convert back to uom
    (Angle::new::<revolution>(clamped_value.0), clamped_value.1)
}

// Note: Unlike [`clamp_revolution`], this function doesn't clamp values; it performs
// a linear scaling even for values outside the range.
pub fn scale_revolution_to_range(value: f64, min: f64, max: f64) -> f64 {
    // we calculate the distance between min and max
    let distance = revolution_distance(min, max);

    // we scale the value to the distance

    (value - min) / distance
}

// Clamps a revolution value to be within the specified range [min, max].
// If the value is outside the range, it will be clamped to either min or max,
// depending on which one it's closer to in the circular context.
pub fn clamp_revolution(value: f64, min: f64, max: f64) -> (f64, Clamping) {
    // normalize value from 0..1
    let value = wrap_revolution(value);
    let min = wrap_revolution(min);
    let max = wrap_revolution(max);

    // check if in acceptable range
    if revolution_in_range(value, min, max) {
        return (value, Clamping::None);
    }

    // calculates the distance between min and max
    let (clamp_to_min_min, clamp_to_min_max, clamp_to_max_min, clamp_to_max_max) =
        clamping_ranges(min, max);

    // check if in min clamping  range
    if revolution_in_range(value, clamp_to_min_min, clamp_to_min_max) {
        return (min, Clamping::Min);
    }
    // check if in max clamping  range
    if revolution_in_range(value, clamp_to_max_min, clamp_to_max_max) {
        return (max, Clamping::Max);
    }

    // at this point our input value should be either retured (cause in spec) or clamped to min or max
    // so this point should never be reached
    // in case it does we just clamp to min
    (min, Clamping::Min)
}

/*
    Calculates the clamping ranges for min and max values in a circular context.
    This is used internally by `clamp_revolution` to determine whether out-of-range
    values should be clamped to the min or max value.
    The clamping strategy divides the out-of-range space into two regions:
    values closer to min are clamped to min, and values closer to max are clamped to max.
*/
fn clamping_ranges(min: f64, max: f64) -> (f64, f64, f64, f64) {
    // normalize min and max
    let min = wrap_revolution(min);
    let max = wrap_revolution(max);
    // calculates the distance between min and max (distance A in the test)
    let in_spec_distance = revolution_distance(min, max);
    // calculate distance B and clamping distance as per the test comment
    let out_spec_distance = 1.0 - in_spec_distance;
    let clamping_distance = out_spec_distance / 2.0;

    let clamp_to_min_min = wrap_revolution(min - clamping_distance);
    let clamp_to_min_max = min;
    let clamp_to_max_min = max;
    let clamp_to_max_max = wrap_revolution(max + clamping_distance);

    (
        clamp_to_min_min,
        clamp_to_min_max,
        clamp_to_max_min,
        clamp_to_max_max,
    )
}

// Calculates the shortest distance between two points in a circular [0,1) range.
// This function properly handles cases where the shortest path crosses the 0/1 boundary.
fn revolution_distance(min: f64, max: f64) -> f64 {
    // Normalize the values to ensure they're in the [0, 1) range
    let normalized_min = wrap_revolution(min);
    let normalized_max = wrap_revolution(max);

    // Check if the range crosses zero
    if normalized_min > normalized_max {
        // For ranges that cross zero (e.g., min = 0.9, max = 0.1)
        // The distance is (1 - min) + max
        1.0 - normalized_min + normalized_max
    } else {
        // For normal ranges (e.g., min = 0.1, max = 0.3)
        // The distance is simply max - min
        normalized_max - normalized_min
    }
}

// Wraps any floating-point value to the [0,1) range, handling the circular nature of revolutions.
// This is useful for normalizing angles or other periodic values that represent
// a full revolution when they reach 1.0.
fn wrap_revolution(value: f64) -> f64 {
    let mut normalized = value % 1.0;
    if normalized == 0.0 && value >= 1.0 {
        return 1.0;
    }
    if normalized < 0.0 {
        normalized += 1.0;
    }
    normalized
}

// Checks if a value is within a specified range in a circular [0,1) context.
// This function properly handles ranges that cross the 0/1 boundary.
fn revolution_in_range(value: f64, min: f64, max: f64) -> bool {
    // check if cross 0 boundary
    let cross_zero = min > max;

    if cross_zero {
        // For ranges that cross zero (e.g., min = 0.9, max = 0.1)
        // value is in range if it's either >= min OR <= max
        if value >= min || value <= max {
            return true;
        }
        false
    } else {
        // check if value is in range for non-crossing ranges
        if value >= min && value <= max {
            return true;
        }
        false
    }
}
