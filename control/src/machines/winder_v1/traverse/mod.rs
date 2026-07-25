use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use control_runtime::machine::Measurement;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;
use qitech_lib::units::angular_velocity::revolution_per_second;
use qitech_lib::units::length::millimeter;
use qitech_lib::units::velocity::millimeter_per_second;

use crate::converters::LinearStepConverter;
use crate::machines::winder_v1::spool::Spool;

mod types;
use types::HomingState;
use types::State;
use types::TraversingState;

pub struct Traverse {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    motor_port: usize,
    endstop_port: usize,

    laser: Rc<RefCell<dyn DigitalOutputDevice>>,

    // --- config ---
    enabled: bool,
    limit_inner: Length,
    limit_outer: Length,
    step_size: Length,
    padding: Length,

    // --- state ---
    state: State,
    laser_enabled: bool,

    // --- measurements ---
    position: Measurement<millimeter>,
    speed: Measurement<millimeter_per_second>,

    // --- converters ---
    fullstep_converter: LinearStepConverter,
    microstep_converter: LinearStepConverter,
}

impl Traverse {
    pub fn update(&mut self, spool: &Spool) {
        if !self.enabled {
            return;
        }

        self.update_position();

        if let Some(next_state) = self.next_state() {
            if matches!(next_state, State::Homing(HomingState::Validate(_))) {
                // Set poition of traverse to 0
                self.device.borrow_mut().set_position(self.motor_port, 0);
            }

            self.state = next_state;
        }

        self.update_speed(spool);
    }
}

// update helpers
impl Traverse {
    fn update_position(&mut self) {
        let steps = self.device.borrow().get_position(self.motor_port);
        let position = self.microstep_converter.steps_to_distance(steps as f64);
        self.position.set(position);
    }

    fn update_speed(&mut self, spool: &Spool) {
        let speed = self.compute_speed(spool);
        let steps_per_second = self.fullstep_converter.velocity_to_steps(speed);
        self.device
            .borrow_mut()
            .set_speed(self.motor_port, steps_per_second);
    }
}

// state transitions
impl Traverse {
    fn next_state(&self) -> Option<State> {
        use State::*;

        match &self.state {
            NotHomed | Idle => None,
            GoingIn => {
                // wait until we reach the inner limit
                if !self.is_at_position(self.limit_inner) {
                    return None;
                }

                Some(State::Idle)
            }
            GoingOut => {
                // wait until we reach the outer limit
                if !self.is_at_position(self.limit_outer) {
                    return None;
                }

                Some(State::Idle)
            }
            Homing(state) => self.next_state_from_homing_state(state),
            Traversing(state) => self.next_state_from_traversing_state(state),
        }
    }

    fn next_state_from_homing_state(&self, homing_state: &HomingState) -> Option<State> {
        use HomingState::*;

        match homing_state {
            Initialize => {
                if self.endstop_triggered() {
                    // If endstop is triggered, escape the endstop
                    Some(State::Homing(EscapeEndstop))
                } else {
                    // If endstop is not triggered, move to the endstop
                    Some(State::Homing(FindEndstopCoarse))
                }
            }
            EscapeEndstop => {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() {
                    return None;
                }

                // now start finding
                Some(State::Homing(FindEndstopFineDistancing))
            }
            FindEndstopFineDistancing => {
                // move out until endstop is not triggered anymore
                if self.endstop_triggered() {
                    return None;
                }

                Some(State::Homing(FindEndstopFine))
            }
            FindEndstopFine => {
                // move to endstop
                if !self.endstop_triggered() {
                    return None;
                }

                // TODO: outside
                // // set poition of traverse to 0
                // self.motor..set_position(0);

                // now validate
                Some(State::Homing(Validate(Instant::now())))
            }
            FindEndstopCoarse => {
                // move to endstop
                if !self.endstop_triggered() {
                    return None;
                }

                // now move away from endstop
                Some(State::Homing(FindEndstopFineDistancing))
            }
            Validate(instant) => {
                if instant.elapsed() <= Duration::from_millis(100) {
                    // as ms
                    return None;
                }

                // should be at zero now
                if self.is_at_position(Length::ZERO) {
                    Some(State::Idle)
                } else {
                    // validation failed. retry
                    Some(State::Homing(Initialize))
                }
            }
        }
    }

    fn next_state_from_traversing_state(&self, state: &TraversingState) -> Option<State> {
        use TraversingState::*;

        match state {
            TraversingIn => {
                // inner limit not reached yet
                if self.position.get() > self.limit_inner + self.padding {
                    return None;
                }

                // now traverse to out
                Some(State::Traversing(TraversingOut))
            }
            GoingOut | TraversingOut => {
                // outer limit not reached yet
                if self.position.get() < self.limit_outer - self.padding {
                    return None;
                }

                // now traverse to in
                Some(State::Traversing(TraversingIn))
            }
        }
    }
}

// speed computation
impl Traverse {
    fn compute_speed(&mut self, spool: &Spool) -> Velocity {
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
                    as_velocity(10.0)
                } else {
                    as_velocity(100.0)
                }
            }
            GoingOut => {
                // Move out at a speed of 10-100 mm/s
                let target = self.limit_outer;

                if self.is_close_to_target(target) {
                    as_velocity(10.0)
                } else {
                    as_velocity(100.0)
                }
            }
            Homing(state) => self.speed_from_homing_state(state),
            Traversing(state) => self.speed_from_traversing_state(state, spool),
        }
    }

    /// yield as mm/s
    fn speed_from_homing_state(&self, homing_state: &HomingState) -> Velocity {
        use HomingState::*;

        match homing_state {
            Initialize => as_velocity(0.0),

            // Move out at a speed of 10 mm/s
            EscapeEndstop => as_velocity(10.0),

            // Move out at a speed of 2 mm/s
            FindEndstopFineDistancing => as_velocity(2.0),

            // Move in at a speed of -100 mm/s
            FindEndstopCoarse => as_velocity(-100.0),

            // move into the endstop at 2 mm/s
            FindEndstopFine => as_velocity(2.0),

            // We stand still until the validation cooldown has passed
            Validate(_) => as_velocity(0.0),
        }
    }

    fn speed_from_traversing_state(
        &self,
        traversing_state: &TraversingState,
        spool: &Spool,
    ) -> Velocity {
        use TraversingState::*;

        let offset = Length::new::<millimeter>(0.01);

        let (target_position, speed) = match traversing_state {
            // Move out at a speed of 100 mm/s initially
            GoingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = as_velocity(100.0);
                (position, speed)
            }
            TraversingIn => {
                let position = self.limit_inner + self.padding - offset;
                let speed = Self::calculate_traverse_speed(spool, self.step_size);
                (position, speed)
            }
            TraversingOut => {
                let position = self.limit_outer - self.padding + offset;
                let speed = Self::calculate_traverse_speed(spool, self.step_size);
                (position, speed)
            }
        };

        self.speed_to_position(target_position, speed)
    }
}

// helpers
impl Traverse {
    fn calculate_traverse_speed(spool: &Spool, step_size: Length) -> Velocity {
        let spool_speed = spool.speed().get::<revolution_per_second>();
        let step_size = step_size.get::<millimeter>();

        // Calculate the traverse speed directly from spool speed and step size
        Velocity::new::<millimeter_per_second>(spool_speed * step_size)
    }

    // Changes the direction of the speed based on the current position and target position
    fn speed_to_position(&self, target_position: Length, absolute_speed: Velocity) -> Velocity {
        // If we are over the target position we need to move negative
        if self.position.get() > target_position {
            -absolute_speed.abs()
        } else if self.position.get() < target_position {
            absolute_speed.abs()
        } else {
            Velocity::ZERO
        }
    }

    fn endstop_triggered(&self) -> bool {
        self.device
            .borrow()
            .get_digital_input(self.endstop_port)
            .unwrap_or(false)
    }

    /// Calculate distance to position
    fn distance_to_position(&self, target_position: Length) -> Length {
        (self.position.get() - target_position).abs()
    }

    fn is_at_position(&self, target_position: Length) -> bool {
        let tolerance = Length::new::<millimeter>(0.01);
        let upper_tolerance = target_position + tolerance.abs();
        let lower_tolerance = target_position - tolerance.abs();
        lower_tolerance <= self.position.get() && self.position.get() <= upper_tolerance
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

/// we always initialize velocity as millimeters per second
/// so this helper reduces the long name and avoids accidentally
/// using the wrong unit.
fn as_velocity(value: f64) -> Velocity {
    Velocity::new::<millimeter_per_second>(value)
}
