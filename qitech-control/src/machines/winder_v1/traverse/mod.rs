use control_runtime::Measurement;
use qitech_lib::ethercat_hal::io::{digital_output::DigitalOutputDevice, stepper_velocity_el70x1::StepperVelocityEL70x1Device};
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::millimeter_per_second;
use qitech_lib::units::{AngularVelocity, ConstZero, Length, Velocity};
use std::{cell::RefCell, rc::Rc};

use crate::converters::linear_step_converter::LinearStepConverter;

mod types;
use types::State;
use types::HomingState;
use types::TraversingState;

mod traverse_controller;

pub struct Traverse {
    // --- hardware ---
    motor: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    laser: Rc<RefCell<dyn DigitalOutputDevice>>,

    // --- config ---
    enabled: bool,

    // --- state ---
    laser_enabled: bool,

    // --- measurements ---
    position: Measurement<Length, millimeter>,

    limit_inner: Length,
    limit_outer: Length,

    step_size: Length,

    padding: Length,

    state: State,

    // --- converters ---
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

impl Traverse {
    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    /// Can go to inner limit capability check
    pub fn can_go_in(&self) -> bool {
        // // Check if traverse is homed, not in standby, not traversing
        // // Allow changing direction (even when going out)
        // // Disallow when homing is in progress
        // self.is_homed()
        //     && self.traverse_mode != TraverseMode::Standby
        //     && !self.traverse_controller.is_going_in()
        //     && !self.traverse_controller.is_going_home()
        //     && !self.traverse_controller.is_traversing()
        //     && self.mode != Winder2Mode::Wind

        false
    }

    /// Can go to outer limit capability check
    pub fn can_go_out(&self) -> bool {
        // // Check if traverse is homed, not in standby, not traversing
        // // Allow changing direction (even when going in)
        // // Disallow when homing is in progress
        // self.traverse_controller.is_homed()
        //     && self.traverse_mode != TraverseMode::Standby
        //     && !self.traverse_controller.is_going_out()
        //     && !self.traverse_controller.is_going_home()
        //     && !self.traverse_controller.is_traversing()
        //     && self.mode != Winder2Mode::Wind

        false
    }

    /// Can go home capability check
    pub fn can_go_home(&self) -> bool {
        // // Check if not in standby, not traversing
        // // Allow going home even when going in or out
        // self.traverse_mode != TraverseMode::Standby
        //     && !self.traverse_controller.is_going_home()
        //     && !self.traverse_controller.is_traversing()
        //     && self.mode != Winder2Mode::Wind

        false
    }
}

// State management
impl Traverse {
    pub const fn goto_limit_inner(&mut self) {
        self.state = State::GoingIn;
    }

    pub const fn goto_limit_outer(&mut self) {
        self.state = State::GoingOut;
    }

    pub const fn goto_home(&mut self) {
        self.state = State::Homing(HomingState::Initialize);
    }

    pub const fn start_traversing(&mut self) {
        self.state = State::Traversing(TraversingState::GoingOut);
    }

    pub const fn is_homed(&self) -> bool {
        // if not [`State::NotHomed`], then it is homed
        !matches!(self.state, State::NotHomed)
    }

    pub const fn is_going_in(&self) -> bool {
        // [`State::GoingIn`]
        matches!(self.state, State::GoingIn)
    }

    pub const fn is_going_out(&self) -> bool {
        // [`State::GoingOut`]
        matches!(self.state, State::GoingOut)
    }

    pub const fn is_going_home(&self) -> bool {
        // [`State::Homing`]
        matches!(self.state, State::Homing(_))
    }

    pub const fn is_traversing(&self) -> bool {
        // [`State::Traversing`]
        matches!(self.state, State::Traversing(_))
    }
}

// velocity computation
impl Traverse {
    fn speed_from_state(&self, spool_speed: AngularVelocity) -> Velocity {
        use State::*;

        match &self.state {
            // Not homed, no movement
            NotHomed => Velocity::ZERO,
            // No movement in idle state
            Idle => Velocity::ZERO,
            GoingIn => {
                // Move in at a speed of 10-100 mm/s
                let target = self.limit_inner;

                if self.is_close_to_target(target) {
                    Velocity::new::<millimeter_per_second>(10.0)
                } else {
                    Velocity::new::<millimeter_per_second>(100.0)
                }
            }
            GoingOut => {
                // Move out at a speed of 10-100 mm/s
                let target = self.limit_outer;

                if self.is_close_to_target(target) {
                    Velocity::new::<millimeter_per_second>(10.0)
                } else {
                    Velocity::new::<millimeter_per_second>(100.0)
                }
            }
            Homing(state) => self.speed_from_homing_state(state),
            Traversing(state) => self.speed_from_traversing_state(state, spool_speed),
        }
    }

    fn speed_from_homing_state(&self, homing_state: &HomingState) -> Velocity {
        use HomingState::*;

        match homing_state {
            Initialize => Velocity::ZERO,

            // Move out at a speed of 10 mm/s
            EscapeEndstop => velocity(10.0),

            // Move out at a speed of 2 mm/s
            FindEndstopFineDistancing => velocity(2.0),

            // Move in at a speed of -100 mm/s
            FindEndstopCoarse => velocity(-100.0),

            // move into the endstop at 2 mm/s
            FindEndstopFine => velocity(2.0),

            // We stand still until the validation cooldown has passed
            Validate(_) => Velocity::ZERO,
        }
    }

    fn speed_from_traversing_state(
        &self,
        traversing_state: TraversingState,
        spool_speed: AngularVelocity,
    ) -> Velocity {
        use TraversingState::*;

        let offset = Length::new::<millimeter>(0.01);

        let (target_position, speed) = match traversing_state {
            // Move out at a speed of 100 mm/s initially
            GoingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = self.config.speed_config.traverse_going_out;
                (position, speed)
            }
            TraversingIn => {
                let position = self.limit_inner + self.padding - offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
            TraversingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = Self::calculate_traverse_speed(spool_speed, self.step_size);
                (position, speed)
            }
        };

        self.speed_to_position(target_position, speed)
    }
}

// helpers
impl Traverse {
    fn update_position(&mut self) {
        let steps = self.motor.get_position() as f64;
        self.position = self.microstep_converter.steps_to_distance(steps);
    }

    fn update_state(&mut self) {
        if let Some(next_state) = self.next_state() {
            self.state = next_state;
        }
    }

    fn update_speed(&mut self, spool_speed: AngularVelocity) {
        let steps_per_second = self.compute_output_steps(spool_speed);
        _ = self.motor.set_speed(steps_per_second);
    }

    fn compute_output_steps(&self, spool_speed: AngularVelocity) -> f64 {
        let speed = self.speed_from_state(spool_speed);
        self.fullstep_converter.velocity_to_steps(speed)
    }

    fn endstop_triggered(&self) -> bool {
        self.limit_switch.get_value().unwrap_or(false)
    }

    fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity {
        let spool_speed = spool_speed.get::<revolution_per_second>();
        let step_size = step_size.get::<millimeter>();

        // Calculate the traverse speed directly from spool speed and step size
        Velocity::new::<millimeter_per_second>(spool_speed * step_size)
    }

    // Changes the direction of the speed based on the current position and target position
    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity {
        // If we are over the target position we need to move negative
        if self.position > target_position {
            -absolute_speed.abs()
        } else if self.position < target_position {
            absolute_speed.abs()
        } else {
            Velocity::ZERO
        }
    }

    /// Calculate distance to position
    fn distance_to_position(&self, target_position: Length) -> Length {
        (self.position.get() - target_position).abs()
    }

    fn is_at_position(&self, target_position: Length) -> bool {
        let tolerance = self.config.length_tolerance;
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        lower_tolerance <= self.position && self.position <= upper_tolerance
    }

    /// Validates that traverse limits maintain proper constraints:
    /// - Inner limit must be smaller than outer limit
    /// - At least 0.9mm difference between inner and outer limits
    fn validate_traverse_limits(inner: Length, outer: Length) -> bool {
        outer > inner + Length::new::<millimeter>(0.9)
    }

    fn is_close_to_target(&self, target: Length) -> bool {
        self.distance_to_position(target).abs() <= Length::new::<millimeter>(1.0)
    }
}

fn velocity(value: f64) -> Velocity {
    Velocity::new::<millimeter_per_second>(value)
}