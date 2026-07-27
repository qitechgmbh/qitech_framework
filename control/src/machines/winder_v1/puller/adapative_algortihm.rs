use std::time::Instant;

use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::length::meter;
use qitech_lib::units::velocity::meter_per_second;

/// Controls adaptive puller speed based on laser diameter feedback.
///
/// # Behaviour
/// - **Inner deadzone** (`accepted_difference`): if `|current − target| ≤
///   accepted_difference` (mm) the measurement is considered close enough to the
///   target and no adjustment is made.  The meter accumulator is reset so the
///   delay always restarts when re-entering this zone.
/// - **Outer boundary** (`lower` / `upper` tolerances from the laser): if the
///   diameter leaves the inner deadzone, meters are accumulated.  After
///   `adjustment_interval_meters` have elapsed the modulation is nudged by
///   ±`step_percent` in the direction that brings the diameter back toward
///   target, and the accumulator is reset.
/// - **Soft limit**: modulation is clamped so the output speed never deviates
///   more than `max_speed_change_percent` % from the base speed.
#[derive(Debug, Clone)]
pub struct SpeedAlgorithmAdaptive {
    // config
    speed_delta_max: f64,
    increase_per_step: f64,
    tolerance_limit: Length,
    adjustment_distance: Length,

    // internal state
    modulation: f64,
    distance_since_last_adjustment: Length,
    time_since_last_update: Instant,
}

impl Default for SpeedAlgorithmAdaptive {
    fn default() -> Self {
        Self {
            speed_delta_max: 0.0,
            increase_per_step: 0.0,
            adjustment_distance: Length::ZERO,
            tolerance_limit: Length::ZERO,
            modulation: 0.0,
            distance_since_last_adjustment: Length::ZERO,
            time_since_last_update: Instant::now(),
        }
    }
}

// public interface
impl SpeedAlgorithmAdaptive {
    pub fn compute(&self, base_speed: Velocity) -> Velocity {
        let factor = 1.0 + self.modulation * self.speed_delta_max;
        (base_speed * factor).max(Velocity::ZERO)
    }

    pub fn update_with_laser_data(
        &mut self,
        now: Instant,
        prev_speed: Velocity,
        current: Length,
        target: Length,
        lower: Length,
        upper: Length,
    ) {
        let dt = now
            .duration_since(self.time_since_last_update)
            .as_secs_f64();

        self.time_since_last_update = now;

        // kept for future use (e.g. trend detection)
        let lower_bound = target - lower;
        let upper_bound = target + upper;
        let _ = (lower_bound, upper_bound);

        // --- Inner deadzone (accepted_difference) ---
        // If the diameter is within ±accepted_difference of the target it is
        // acceptable.  Reset the accumulator so the delay always starts fresh.
        if (current - target).abs() <= self.tolerance_limit {
            self.distance_since_last_adjustment = Length::ZERO;
            return;
        }

        // --- Accumulate metres ---
        let meters_added = prev_speed.abs().get::<meter_per_second>() * dt;
        self.distance_since_last_adjustment += Length::new::<meter>(meters_added);

        // --- Wait for the interval to elapse ---
        if self.distance_since_last_adjustment < self.adjustment_distance {
            return;
        }

        // --- Apply one step in the required direction ---
        // Diameter too large -> speed up the puller (positive modulation)
        // Diameter too small -> slow down the puller (negative modulation)
        let correction_sign: f64 = if current > target { 1.0 } else { -1.0 };
        let step = self.increase_per_step * correction_sign;
        self.modulation = (self.modulation + step).clamp(-1.0, 1.0);
        self.distance_since_last_adjustment = Length::ZERO;
    }
}

// getters + setters
impl SpeedAlgorithmAdaptive {
    pub fn speed_delta_max(&self) -> f64 {
        self.speed_delta_max
    }

    pub fn set_speed_delta_max(&mut self, value: f64) {
        self.speed_delta_max = value.max(0.0);
    }

    pub fn increase_per_step(&self) -> f64 {
        self.increase_per_step
    }

    pub fn set_increase_per_step(&mut self, value: f64) {
        self.increase_per_step = value.max(0.0).min(1.0);
    }

    pub fn adjustment_distance(&self) -> Length {
        self.adjustment_distance
    }

    pub fn set_adjustment_distance(&mut self, value: Length) {
        self.adjustment_distance = value.max(Length::ZERO);
    }

    pub fn tolerance_limit(&self) -> Length {
        self.tolerance_limit
    }

    pub fn set_tolerance_limit(&mut self, value: Length) {
        self.tolerance_limit = value.max(Length::ZERO);
    }

    /// Current modulation level in [-1.0, 1.0].
    pub fn modulation(&self) -> f64 {
        self.modulation
    }

    /// Reset modulation to zero so the algorithm starts fresh from the base speed.
    pub fn reset_modulation(&mut self) {
        self.modulation = 0.0;
        self.distance_since_last_adjustment = Length::ZERO;
    }
}
