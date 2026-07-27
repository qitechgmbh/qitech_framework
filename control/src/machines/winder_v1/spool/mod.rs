use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_framework::uom::AngularVelocity;

use crate::converters::AngularStepConverter;
use crate::machines::winder_v1::Puller;
use crate::machines::winder_v1::TensionArm;
use crate::types::RotationDirection;

mod speed_controller_min_max;
use speed_controller_min_max::SpeedControllerMinMax;

mod speed_controller_adaptive;
use speed_controller_adaptive::SpeedControllerAdaptive;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum SpeedControlMode {
    #[default]
    Adaptive,
    MinMax,
}

pub struct Spool {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device + 'static>>,
    device_port: usize,

    // --- other ---
    enabled: bool,
    direction: RotationDirection,
    speed_control_mode: SpeedControlMode,

    // --- speed controllers ---
    speed_controller_min_max: SpeedControllerMinMax,
    speed_controller_adaptive: SpeedControllerAdaptive,

    // --- converters ---
    step_converter: AngularStepConverter,
}

impl Spool {
    pub fn speed(&self) -> AngularVelocity {
        match self.speed_control_mode {
            SpeedControlMode::MinMax => self.speed_controller_min_max.speed(),
            SpeedControlMode::Adaptive => self.speed_controller_adaptive.speed(),
        }
    }

    pub fn update(&mut self, t: Instant, tension_arm: &TensionArm, puller: &Puller) {
        self.update_speed(t, tension_arm, puller);
        self.sync_hardware();
    }
}

// --- helpers ---
impl Spool {
    fn update_speed(&mut self, t: Instant, tension_arm: &TensionArm, puller: &Puller) {
        match self.speed_control_mode {
            SpeedControlMode::MinMax => self.speed_controller_min_max.update(t, tension_arm),
            SpeedControlMode::Adaptive => {
                self.speed_controller_adaptive
                    .update(t, tension_arm, puller)
            }
        }
    }

    fn sync_hardware(&mut self) {
        // retrieve current speed
        let angular_velocity = match self.speed_control_mode {
            SpeedControlMode::MinMax => self.speed_controller_min_max.speed_clamped(),
            SpeedControlMode::Adaptive => self.speed_controller_adaptive.speed_clamped(),
        };

        // apply sign
        let angular_velocity = match self.direction {
            RotationDirection::Forward => angular_velocity,
            RotationDirection::Reverse => -angular_velocity,
        };

        // convert into steps
        let steps_per_second = self
            .step_converter
            .angular_velocity_to_steps(angular_velocity);

        // write
        let mut device = self.device.borrow_mut();
        _ = device.set_speed(self.device_port, steps_per_second);
    }
}

// --- callbacks ---
impl Spool {
    pub fn on_speed_control_mode_changed(&mut self) {
        // Get the current speed from the active controller
        let current_speed = match self.speed_control_mode {
            SpeedControlMode::Adaptive => self.speed_controller_adaptive.speed(),
            SpeedControlMode::MinMax => self.speed_controller_min_max.speed(),
        };

        // Set the speed in the target controller and reset it for smooth transition
        match self.speed_control_mode {
            SpeedControlMode::Adaptive => {
                self.speed_controller_adaptive.set_speed(current_speed);
                self.speed_controller_adaptive.reset();

                // Set again after reset to maintain speed
                self.speed_controller_adaptive.set_speed(current_speed);
            }
            SpeedControlMode::MinMax => {
                self.speed_controller_min_max.set_speed(current_speed);
                self.speed_controller_min_max.reset();

                // Set again after reset to maintain speed
                self.speed_controller_min_max.set_speed(current_speed);
            }
        }
    }
}
