use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
use qitech_lib::units::ConstZero;
use qitech_lib::units::Length;
use qitech_lib::units::Velocity;

use crate::controllers::LinearJerkSpeedController;
use crate::converters::LinearStepConverter;
use crate::types::RotationDirection;

mod types;
use types::GearRatio;
use types::SpeedRegulationMode;

mod adapative_algortihm;
use adapative_algortihm::SpeedAlgorithmAdaptive;

pub struct Puller {
    // --- hardware ---
    device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
    device_port: usize,

    // --- config ---
    direction: RotationDirection,
    gear_ratio: GearRatio,
    speed_target: Velocity,
    speed_regulation_mode: SpeedRegulationMode,

    // --- state ---
    enabled: bool,

    // --- measurements ---
    speed: Velocity,

    // --- converters ----
    step_converter: LinearStepConverter,

    // --- controllers ----
    acceleration_controller: LinearJerkSpeedController,

    // --- speed algortihms ---
    speed_algorithm_adaptive: SpeedAlgorithmAdaptive,
}

impl Puller {
    pub fn new(
        // --- hardware ---
        device: Rc<RefCell<dyn StepperVelocityEL70x1Device>>,
        device_port: usize,
        // --- config ---
        direction: RotationDirection,
        gear_ratio: GearRatio,
        speed_target: Velocity,
        speed_regulation_mode: SpeedRegulationMode,
        // --- measurements ---
        speed: Velocity,
        // --- converters ----
        step_converter: LinearStepConverter,
        // --- controllers ----
        acceleration_controller: LinearJerkSpeedController,
        // --- speed algortihms ---
        speed_algorithm_adaptive: SpeedAlgorithmAdaptive,
    ) -> Self {
        Self {
            enabled: false,
            device,
            device_port,
            direction,
            gear_ratio,
            speed_target,
            speed_regulation_mode,
            speed,
            step_converter,
            acceleration_controller,
            speed_algorithm_adaptive,
        }
    }

    pub fn speed(&self) -> Velocity {
        self.speed
    }

    pub fn update(&mut self, t: Instant) {
        self.update_speed(t);
        self.sync_hardware();
    }

    pub fn update_with_laser_data(
        &mut self,
        now: Instant,
        current: Length,
        target: Length,
        lower: Length,
        upper: Length,
    ) {
        self.speed_algorithm_adaptive.update_with_laser_data(
            now,
            self.speed(),
            current,
            target,
            lower,
            upper,
        );
    }
}

// --- helpers ---
impl Puller {
    fn update_speed(&mut self, t: Instant) {
        let base_speed = if self.enabled {
            use SpeedRegulationMode::*;
            match self.speed_regulation_mode {
                Speed => self.speed_target,
                AdaptiveDiameter => self.speed_algorithm_adaptive.compute(self.speed_target),
            }
        } else {
            Velocity::ZERO
        };

        let speed = base_speed * self.gear_ratio.multiplier() * self.direction.modifier();
        self.speed = self.acceleration_controller.update(speed, t);
    }

    fn sync_hardware(&mut self) {
        let angular_velocity = self.step_converter.velocity_to_angular_velocity(self.speed);

        let steps_per_second = self
            .step_converter
            .angular_velocity_to_steps(angular_velocity);

        let mut device = self.device.borrow_mut();
        _ = device.set_speed(self.device_port, steps_per_second);
    }
}
