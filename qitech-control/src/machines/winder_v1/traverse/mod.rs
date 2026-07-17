use qitech_lib::ethercat_hal::io::{digital_output::DigitalOutputDevice, stepper_velocity_el70x1::StepperVelocityEL70x1Device};
use qitech_lib::units::{AngularVelocity, ConstZero, Velocity};
use std::{cell::RefCell, rc::Rc};

use crate::converters::linear_step_converter::LinearStepConverter;

mod types;
use types::State;
use types::HomingState;
use types::TraversingState;

pub struct Traverse {
    // --- hardware ---
    motor: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    laser: Rc<RefCell<dyn DigitalOutputDevice>>,

    // --- config ---
    enabled: bool,

    // --- state ---
    laser_enabled: bool,

    // --- measurements ---


    position: Length,
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
        // Check if traverse is homed, not in standby, not traversing
        // Allow changing direction (even when going out)
        // Disallow when homing is in progress
        self.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_in()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
    }

    /// Can go to outer limit capability check
    pub fn can_go_out(&self) -> bool {
        // Check if traverse is homed, not in standby, not traversing
        // Allow changing direction (even when going in)
        // Disallow when homing is in progress
        self.traverse_controller.is_homed()
            && self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_out()
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
    }

    /// Can go home capability check
    pub fn can_go_home(&self) -> bool {
        // Check if not in standby, not traversing
        // Allow going home even when going in or out
        self.traverse_mode != TraverseMode::Standby
            && !self.traverse_controller.is_going_home()
            && !self.traverse_controller.is_traversing()
            && self.mode != Winder2Mode::Wind
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

        match self.state {
            // Not homed, no movement
            NotHomed => Velocity::ZERO,
            // No movement in idle state
            Idle => Velocity::ZERO,
            GoingIn => {
                let position = self.limit_inner;
                let speed = match self.is_close_to_target(position) {
                    true => self.config.speed_config.move_close,
                    false => self.config.speed_config.move_not_close,
                };

                self.speed_to_position(position, speed)
            }
            GoingOut => {
                let position = self.limit_outer;
                let speed = match self.is_close_to_target(position) {
                    true => self.config.speed_config.move_close,
                    false => self.config.speed_config.move_not_close,
                };

                self.speed_to_position(position, speed)
            }
            Homing(state) => self.speed_from_homing_state(state),
            Traversing(state) => self.speed_from_traversing_state(state, spool_speed),
        }
    }

    fn speed_from_homing_state(&self, homing_state: HomingState) -> Velocity {
        use HomingState::*;

        let sc = &self.config.speed_config;

        match homing_state {
            Initialize => Velocity::ZERO,
            EscapeEndstop => sc.homing_escape_end_stop,
            FindEndstopFineDistancing => sc.homing_find_endstop_fine_distancing,
            FindEndstopCoarse => sc.homing_find_endstop_coarse,
            FindEndstopFine => sc.homing_find_endstop_fine,
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