use std::time::Instant;

use qitech_framework::uom::Angle;
use qitech_framework::uom::AngularAcceleration;
use qitech_framework::uom::AngularVelocity;
use qitech_framework::uom::ConstZero;
use qitech_framework::uom::angle::degree;
use qitech_framework::uom::angular_acceleration::radian_per_second_squared;
use qitech_framework::uom::angular_acceleration::revolution_per_minute_per_second;
use qitech_framework::uom::angular_velocity::radian_per_second;
use qitech_framework::uom::angular_velocity::revolution_per_minute;

use super::super::utils::FilamentTensionCalculator;
use crate::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController;
use crate::controllers::second_degree_motion::acceleration_position_controller::MotionControllerError;
use crate::machines::winder_v1::tension_arm::TensionArm;
use crate::machines::winder_v1::utils::clamp_revolution::Clamping;
use crate::machines::winder_v1::utils::clamp_revolution::clamp_revolution_uom;
use crate::utils::interpolation::interpolate_exponential;
use crate::utils::interpolation::scale;
use crate::utils::moving_time_window::MovingTimeWindow;

#[derive(Debug)]
pub struct SpeedControllerMinMax {
    /// Current speed in
    speed: AngularVelocity,
    /// Whether the speed controller is enabled or not
    enabled: bool,
    /// Acceleration controller to dampen speed change
    acceleration_controller: AngularAccelerationSpeedController,
    /// Filament tension calculator
    filament_calc: FilamentTensionCalculator,
    /// Unit is angular velocity in rad/s
    speed_time_window: MovingTimeWindow<f64>,
}

impl Default for SpeedControllerMinMax {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeedControllerMinMax {
    pub fn new() -> Self {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(150.0);

        Self {
            speed: AngularVelocity::ZERO,
            enabled: false,
            acceleration_controller: AngularAccelerationSpeedController::new(
                Some(AngularVelocity::ZERO),
                Some(max_speed),
                -AngularAcceleration::ZERO, // Will be dynamically adjusted
                AngularAcceleration::ZERO,  // Will be dynamically adjusted
                AngularVelocity::ZERO,
            ),
            filament_calc: FilamentTensionCalculator::new(
                Angle::new::<degree>(90.0),
                Angle::new::<degree>(20.0),
            ),
            speed_time_window: MovingTimeWindow::new(
                std::time::Duration::from_secs(5),
                10, // max samples
            ),
        }
    }
}

impl SpeedControllerMinMax {
    /// Helper method to get min speed without Option type
    fn min_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_min_speed()
            .unwrap_or(AngularVelocity::ZERO)
    }

    /// Helper method to get max speed without Option type  
    fn max_speed(&self) -> AngularVelocity {
        self.acceleration_controller
            .get_max_speed()
            .unwrap_or(AngularVelocity::new::<radian_per_second>(f64::INFINITY))
    }

    /// Calculates the desired speed based on the tension arm angle.
    /// If the arm is over it's maximum angle, the speed is set to the minimum speed.
    /// If the arm is under it's minimum angle, the speed is set to the maximum speed.
    /// If the arm is within the range, the speed is interpolated between the minimum and maximum speed based on the tension arm angle.
    fn speed_raw(&mut self, _t: Instant, tension_arm: &TensionArm) -> AngularVelocity {
        let min_speed = AngularVelocity::ZERO;

        // Convert puller speed to angular velocity using a magic factor
        // The factor is adjusted so that the tension arm is reasonably high when a standard spool is at its lowest diameter

        // Respect the configured max speed limits by taking the minimum of dynamic calculation and configured max
        let max_speed = self.max_speed();

        // calculate filament tension
        let tension_arm_angle = tension_arm.angle().unwrap();
        let tension_arm_revolution = clamp_revolution_uom(
            tension_arm_angle,
            // inverted because min angle is max tension
            self.filament_calc.get_max_angle(),
            self.filament_calc.get_min_angle(),
        );

        match tension_arm_revolution.1 {
            Clamping::Min => return min_speed,
            Clamping::Max => return min_speed,
            _ => {}
        };

        let filament_tension = self
            .filament_calc
            .calc_filament_tension(tension_arm_revolution.0);

        let filament_tension_inverted = 1.0 - filament_tension;

        // use exponetial interpolation to make the speed change more sensitive in the lower range
        let filament_tension_exponential = interpolate_exponential(filament_tension_inverted, 2.0);

        // interpolate speed linear

        // save speed
        AngularVelocity::new::<radian_per_second>(scale(
            filament_tension_exponential,
            min_speed.get::<radian_per_second>(),
            max_speed.get::<radian_per_second>(),
        ))
    }

    /// Accelerates the speed using the acceleration controller.
    ///
    /// Parameters:
    /// - `speed`: The current speed
    /// - `t`: The current time.
    ///
    /// Returns:
    /// - The new speed after applying acceleration.
    fn accelerate_speed(&mut self, speed: AngularVelocity, t: Instant) -> AngularVelocity {
        // The min/mac acceleration depends on the max speed of the last 5secs or the target speed (whatever is higher)
        let acceleration = AngularAcceleration::new::<radian_per_second_squared>(
            self.speed_time_window
                .max()
                .abs()
                .max(speed.get::<radian_per_second>().abs())
                // The magic factor is dependent on the scceleration settings on the puller speed controller to reduce oscillation
                * 0.5,
        );

        // Set the acceleration to the controller
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);

        let new_speed = self.acceleration_controller.update(speed, t);

        // add new speed to the time window
        self.speed_time_window
            .update(new_speed.get::<radian_per_second>(), t);

        new_speed
    }
}

impl SpeedControllerMinMax {
    pub fn update(&mut self, t: Instant, tension_arm: &TensionArm) {
        let speed = self.speed_raw(t, tension_arm);
        let speed = match self.enabled {
            true => speed,
            false => AngularVelocity::ZERO,
        };

        self.speed = self.accelerate_speed(speed, t);
    }

    pub fn speed_clamped(&self) -> AngularVelocity {
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();

        if self.speed < min_speed {
            AngularVelocity::ZERO
        } else if self.speed > max_speed {
            max_speed
        } else {
            self.speed
        }
    }

    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn reset(&mut self) {
        self.speed = AngularVelocity::ZERO;
        self.acceleration_controller.reset(AngularVelocity::ZERO);
    }

    fn update_acceleration(&mut self) -> Result<(), MotionControllerError> {
        // Set acceleration to 1/4 of the range between min and max speed
        // The spool will accelerate from min to max speed in 4 seconds
        let min_speed = self.min_speed();
        let max_speed = self.max_speed();
        let range = max_speed - min_speed;
        let acceleration = AngularAcceleration::new::<revolution_per_minute_per_second>(
            range.get::<revolution_per_minute>() / 4.0,
        );
        self.acceleration_controller
            .set_max_acceleration(acceleration);
        self.acceleration_controller
            .set_min_acceleration(-acceleration);
        Ok(())
    }

    pub fn set_max_speed(
        &mut self,
        max_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.acceleration_controller.set_max_speed(Some(max_speed));
        self.update_acceleration()?;
        Ok(())
    }

    pub fn set_min_speed(
        &mut self,
        min_speed: AngularVelocity,
    ) -> Result<(), MotionControllerError> {
        self.acceleration_controller.set_min_speed(Some(min_speed));
        self.update_acceleration()?;
        Ok(())
    }

    pub fn get_max_speed(&self) -> AngularVelocity {
        self.max_speed()
    }

    pub fn get_min_speed(&self) -> AngularVelocity {
        self.min_speed()
    }

    pub fn speed(&self) -> AngularVelocity {
        self.speed
    }

    pub fn set_speed(&mut self, speed: AngularVelocity) {
        self.speed = speed;
        // Also update the acceleration controller's current speed to ensure smooth transitions
        self.acceleration_controller.reset(speed);
    }
}
