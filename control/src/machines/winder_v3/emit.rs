use std::cell::RefMut;
pub use std::time::Instant;

use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
pub use qitech_lib::units::Velocity;
#[cfg(not(feature = "mock-machine"))]
pub use qitech_lib::units::angle::degree;
#[cfg(not(feature = "mock-machine"))]
pub use qitech_lib::units::angular_velocity::revolution_per_minute;
#[cfg(not(feature = "mock-machine"))]
pub use qitech_lib::units::f64::*;
#[cfg(not(feature = "mock-machine"))]
pub use qitech_lib::units::length::meter;
#[cfg(not(feature = "mock-machine"))]
pub use qitech_lib::units::length::millimeter;
pub use qitech_lib::units::velocity::meter_per_minute;

use super::LASER_PORT;
use super::PULLER_PORT;
use super::SPOOL_PORT;
use super::TRAVERSE_PORT;
use super::TraverseMode;
use super::Winder2;
use super::Winder2Mode;
pub use super::api::Measurements;
pub use super::api::ModeState;
use super::api::PullerRegulationMode;
pub use super::api::PullerState;
pub use super::api::SpoolAutomaticActionMode;
pub use super::api::SpoolAutomaticActionState;
pub use super::api::SpoolSpeedControllerState;
pub use super::api::States;
pub use super::api::TensionArmState;
pub use super::api::TraverseState;
use super::spool_speed_controller;

impl Winder2 {
    /// Implement Spool
    /// called by `act`
    pub fn sync_spool_speed(&mut self, t: Instant) {
        let angular_velocity = self.spool_speed_controller.update_speed(
            t,
            &self.tension_arm,
            &self.puller_speed_controller,
        );

        // Apply direction based on forward setting
        let directed_angular_velocity = if self.spool_speed_controller.get_forward() {
            angular_velocity
        } else {
            -angular_velocity
        };

        let steps_per_second = self
            .spool_step_converter
            .angular_velocity_to_steps(directed_angular_velocity);
        let spool_ref = &mut *self.spool.borrow_mut();
        let _ = spool_ref.set_speed(SPOOL_PORT, steps_per_second);
    }

    pub fn stop_or_pull_spool(&mut self, now: Instant) {
        if matches!(
            self.spool_automatic_action.mode,
            SpoolAutomaticActionMode::NoAction
        ) {
            self.calculate_spool_auto_progress_(now);
            return;
        }

        match self.mode {
            Winder2Mode::Pull => self.calculate_spool_auto_progress_(now),
            Winder2Mode::Wind => self.calculate_spool_auto_progress_(now),
            _ => {
                self.spool_automatic_action.progress_last_check = now;
                return;
            }
        }

        if self.spool_automatic_action.progress >= self.spool_automatic_action.target_length {
            match self.spool_automatic_action.mode {
                SpoolAutomaticActionMode::NoAction => (),
                SpoolAutomaticActionMode::Pull => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(&Winder2Mode::Pull);
                }
                SpoolAutomaticActionMode::Hold => {
                    self.stop_or_pull_spool_reset(now);
                    self.set_mode(&Winder2Mode::Hold);
                }
            }
        }
    }
    /// Implement Mode
    pub fn set_mode(&mut self, mode: &Winder2Mode) {
        let should_update = *mode != Winder2Mode::Wind || self.can_wind();

        if should_update {
            // all transitions are allowed
            self.mode = mode.clone();

            // Apply the mode changes to the spool and puller
            self.set_spool_mode(mode);
            self.set_puller_mode(mode);
            self.set_traverse_mode(mode);
        }
    }

    fn get_laser(&mut self) -> RefMut<'_, dyn DigitalOutputDevice> {
        self.laser.borrow_mut()
    }

    /// Implement Traverse
    pub fn set_laser(&mut self, value: bool) {
        self.laser_enabled = value;
        let mut laser = self.get_laser();
        laser.set_output(LASER_PORT, value);
        drop(laser);
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) {
        let new_inner = Length::new::<millimeter>(limit);
        let current_outer = self.traverse_controller.get_limit_outer();

        // Validate the new inner limit against current outer limit
        if !Self::validate_traverse_limits(new_inner, current_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }
        self.traverse_controller.set_limit_inner(new_inner);
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) {
        let new_outer = Length::new::<millimeter>(limit);
        let current_inner = self.traverse_controller.get_limit_inner();

        // Validate the new outer limit against current inner limit
        if !Self::validate_traverse_limits(current_inner, new_outer) {
            // Don't update if validation fails - keep the current value
            return;
        }

        self.traverse_controller.set_limit_outer(new_outer);
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) {
        let step_size = Length::new::<millimeter>(step_size);
        self.traverse_controller.set_step_size(step_size);
    }

    pub fn traverse_set_padding(&mut self, padding: f64) {
        let padding = Length::new::<millimeter>(padding);
        self.traverse_controller.set_padding(padding);
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        if self.can_go_in() {
            self.traverse_controller.goto_limit_inner();
        }
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        if self.can_go_out() {
            self.traverse_controller.goto_limit_outer();
        }
    }

    pub fn traverse_goto_home(&mut self) {
        if self.can_go_home() {
            self.traverse_controller.goto_home();
        }
    }

    pub fn update_measurements(&mut self) {
        let angle_deg = self.tension_arm.get_angle().unwrap();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let angle_deg = if angle_deg >= Angle::new::<degree>(270.0) {
            angle_deg - Angle::new::<degree>(360.0)
        } else {
            angle_deg
        };

        let puller_ref = &mut *self.puller.borrow_mut();

        // Calculate puller speed from current motor steps
        let steps_per_second = puller_ref.get_speed(PULLER_PORT);
        let angular_velocity = self
            .puller_speed_controller
            .converter
            .steps_to_angular_velocity(steps_per_second as f64);
        let motor_speed = self
            .puller_speed_controller
            .angular_velocity_to_speed(angular_velocity);

        // Divide by gear ratio to get actual puller/material speed
        let puller_speed = motor_speed / self.puller_speed_controller.get_gear_ratio().multiplier();
        let spool_ref = &mut *self.spool.borrow_mut();
        // Calculate spool RPM from current motor steps (always positive regardless of direction)
        let spool_rpm = self
            .spool_step_converter
            .steps_to_angular_velocity(spool_ref.get_speed(SPOOL_PORT) as f64)
            .abs();

        // --- write now ---
        self.measurements
            .traverse_position
            .set(self.traverse_controller.get_current_position());

        self.measurements.puller_speed.set(puller_speed.abs());
        self.measurements.spool_rpm.set(spool_rpm);
        self.measurements.tension_arm_angle.set(angle_deg);
        self.measurements
            .spool_progress
            .set(self.spool_automatic_action.progress);
    }

    pub fn update_states(&mut self) {
        // --- update traverse states ---
        let s = &mut self.states.traverse_state;

        s.limit_inner
            .set(self.traverse_controller.get_limit_inner());
        s.limit_outer
            .set(self.traverse_controller.get_limit_outer());

        s.is_going_in.set(self.traverse_controller.is_going_in());
        s.is_going_out.set(self.traverse_controller.is_going_out());
        s.is_homed.set(self.traverse_controller.is_homed());
        s.is_traversing
            .set(self.traverse_controller.is_traversing());
        s.laserpointer.set(self.laser_enabled);
        s.step_size.set(self.traverse_controller.get_step_size());
        s.padding.set(self.traverse_controller.get_padding());

        s.can_go_in.set(self.can_go_in());
        s.can_go_out.set(self.can_go_out());
        s.can_go_home.set(self.can_go_home());

        // --- update puller state ---
        let s = &mut self.states.puller_state;

        s.regulation
            .set(self.puller_speed_controller.regulation_mode.clone());

        s.target_speed
            .set(self.puller_speed_controller.target_speed);

        s.forward.set(self.puller_speed_controller.forward);
        s.gear_ratio.set(self.puller_speed_controller.gear_ratio);

        s.adaptive_speed_delta_max
            .set(self.puller_speed_controller.adaptive.speed_delta_max());

        s.adaptive_adjustment_distance
            .set(self.puller_speed_controller.adaptive.adjustment_distance());

        s.adaptive_change_per_step
            .set(self.puller_speed_controller.adaptive.increase_per_step());

        s.allowed_diameter_deviation
            .set(self.puller_speed_controller.adaptive.tolerance_limit());

        // --- update mode state ---
        let s = &mut self.states.mode_state;

        // s.mode.set();
        s.can_wind.set(self.can_wind());

        // --- update tension arm state ---
        let s = &mut self.states.tension_arm_state;
        s.zeroed.set(self.tension_arm.zeroed);

        // --- update spool speed controller state ---
        let s = &mut self.states.spool_speed_controller_state;

        s.regulation_mode
            .set(self.spool_speed_controller.get_type().clone());

        s.minmax_min_speed
            .set(self.spool_speed_controller.get_minmax_min_speed());

        s.minmax_max_speed
            .set(self.spool_speed_controller.get_minmax_max_speed());

        s.adaptive_tension_target.set(self.spool_speed_controller.get_adaptive_tension_target());

        s.adaptive_radius_learning_rate.set(self
                    .spool_speed_controller
                    .get_adaptive_radius_learning_rate());

        s.adaptive_max_speed_multiplier.set(self
                    .spool_speed_controller
                    .get_adaptive_max_speed_multiplier());

        s.adaptive_acceleration_factor.set(self
                    .spool_speed_controller
                    .get_adaptive_acceleration_factor());

        s.adaptive_deacceleration_urgency_multiplier.set(self
                    .spool_speed_controller
                    .get_adaptive_deacceleration_urgency_multiplier());

        s.forward.set(self.spool_speed_controller.get_forward());

        // --- update spool automatic action state --- 
        let s = &mut self.states.spool_automatic_action_state;

        s.spool_required_meters.set(self.spool_automatic_action.target_length);
        s.spool_automatic_action_mode.set(self.spool_automatic_action.mode.clone());
    }

    /// Apply the mode changes to the spool
    ///
    /// It contains a transition matrix for atomic changes.
    /// It will set [`Self::spool_mode`]
    fn set_traverse_mode(&mut self, mode: &Winder2Mode) {
        // Convert to `Winder2Mode` to `TraverseMode`
        let mode: TraverseMode = mode.clone().into();
        // If coming out of standby
        if self.traverse_mode == TraverseMode::Standby && mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            traverse_ref.set_enabled(TRAVERSE_PORT, true);
            self.traverse_controller.set_enabled(true);
            drop(traverse);
        }

        // If going into standby
        if mode == TraverseMode::Standby && self.traverse_mode != TraverseMode::Standby {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // If we are going into standby, we need to stop the traverse
            traverse_ref.set_enabled(TRAVERSE_PORT, false);
            self.traverse_controller.set_enabled(false);
            drop(traverse);
        }

        {
            let mut traverse = self.traverse.borrow_mut();
            let traverse_ref = &mut *traverse;
            // Transition matrix
            match self.traverse_mode {
                TraverseMode::Standby => match mode {
                    TraverseMode::Standby => {}
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Hold`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Standby`] to [`TraverseMode::Wind`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, true);
                        self.traverse_controller.set_enabled(true);
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Hold => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {}
                    TraverseMode::Traverse => {
                        // From [`TraverseMode::Hold`] to [`TraverseMode::Wind`]
                        self.traverse_controller.start_traversing();
                    }
                },
                TraverseMode::Traverse => match mode {
                    TraverseMode::Standby => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Standby`]
                        traverse_ref.set_enabled(TRAVERSE_PORT, false);
                        self.traverse_controller.set_enabled(false);
                    }
                    TraverseMode::Hold => {
                        // From [`TraverseMode::Wind`] to [`TraverseMode::Hold`]
                        self.traverse_controller.goto_home();
                    }
                    TraverseMode::Traverse => {}
                },
            }
        }

        // Update the internal state
        self.traverse_mode = mode;
    }

    /// Implement Tension Arm
    pub fn tension_arm_zero(&mut self) {
        self.tension_arm.zero();
    }

    pub fn set_spool_automatic_required_meters(&mut self, meters: f64) {
        self.spool_automatic_action.target_length = Length::new::<meter>(meters);
    }

    pub fn set_spool_automatic_mode(&mut self, mode: SpoolAutomaticActionMode) {
        self.spool_automatic_action.mode = mode;
    }

    pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
        self.puller_speed_controller
            .set_regulation_mode(puller_regulation_mode);
    }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller_speed_controller.set_target_speed(target_speed);
    }

    /// Set forward direction
    pub fn puller_set_forward(&mut self, forward: bool) {
        self.puller_speed_controller.set_forward(forward);
    }

    /// Set gear ratio for winding speed
    pub fn puller_set_gear_ratio(&mut self, gear_ratio: super::puller_speed_controller::GearRatio) {
        self.puller_speed_controller.set_gear_ratio(gear_ratio);
    }

    // Spool Speed Controller API methods
    pub fn spool_set_regulation_mode(
        &mut self,
        regulation_mode: spool_speed_controller::SpoolSpeedControllerType,
    ) {
        self.spool_speed_controller.set_type(regulation_mode);
    }

    /// Set minimum speed for minmax mode in RPM
    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) {
        let min_speed = AngularVelocity::new::<revolution_per_minute>(min_speed_rpm);
        if let Err(_) = self.spool_speed_controller.set_minmax_min_speed(min_speed) {
            // tracing::error!("Failed to set spool min speed: {:?}", e);
        }
    }

    /// Set maximum speed for minmax mode in RPM
    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(max_speed_rpm);
        if let Err(_) = self.spool_speed_controller.set_minmax_max_speed(max_speed) {
            // tracing::error!("Failed to set spool max speed: {:?}", e);
        }
    }

    /// Set tension target for adaptive mode (0.0-1.0)
    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) {
        self.spool_speed_controller
            .set_adaptive_tension_target(tension_target);
    }

    /// Set radius learning rate for adaptive mode
    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool_speed_controller
            .set_adaptive_radius_learning_rate(radius_learning_rate);
    }

    /// Set max speed multiplier for adaptive mode
    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool_speed_controller
            .set_adaptive_max_speed_multiplier(max_speed_multiplier);
    }

    /// Set acceleration factor for adaptive mode
    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool_speed_controller
            .set_adaptive_acceleration_factor(acceleration_factor);
    }

    /// Set deacceleration urgency multiplier for adaptive mode
    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool_speed_controller
            .set_adaptive_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
    }

    /// Set forward rotation direction
    pub fn spool_set_forward(&mut self, forward: bool) {
        self.spool_speed_controller.set_forward(forward);
    }
}

// Winder2 Extension
#[cfg(not(feature = "mock-machine"))]
impl Winder2 {
    pub fn puller_set_adaptive_max_speed_change_percent(&mut self, value: f64) {
        self.puller_speed_controller
            .adaptive
            .set_speed_delta_max(value);
    }

    pub fn puller_set_adaptive_adjustment_interval_meters(&mut self, value: f64) {
        self.puller_speed_controller
            .adaptive
            .set_adjustment_distance(Length::new::<meter>(value));
    }

    pub fn puller_set_adaptive_step_percent(&mut self, value: f64) {
        self.puller_speed_controller
            .adaptive
            .set_increase_per_step(value);
    }

    pub fn puller_set_adaptive_accepted_difference(&mut self, value: f64) {
        self.puller_speed_controller
            .adaptive
            .set_tolerance_limit(Length::new::<millimeter>(value));
    }
}
