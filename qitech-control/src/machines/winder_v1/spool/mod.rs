use std::mem;

use super::{
    adaptive_spool_speed_controller::AdaptiveSpoolSpeedController, 
    minmax_spool_speed_controller::MinMaxSpoolSpeedController
};

enum RotationDirection {
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Default)]
enum SpeedControlMode {
    #[default]
    Adaptive,
    MinMax,
}

pub struct Spool {
    enabled: bool,
    direction: RotationDirection,
    speed_mode: SpeedControlMode,

    // --- speed controllers ---
    adaptive_controller: AdaptiveSpoolSpeedController,
    minmax_controller: MinMaxSpoolSpeedController,
}

impl Spool {
    pub fn switch_mode(&mut self, mode: SpeedControlMode) {
        if mem::discriminant(&self.speed_mode) == mem::discriminant(&mode) {
            // identical mode, simply return
            return;
        }

        // Get the current speed from the active controller
        let current_speed = match self.speed_mode {
            SpeedControlMode::Adaptive => self.adaptive_controller.get_speed(),
            SpeedControlMode::MinMax => self.minmax_controller.get_speed(),
        };

        // Set the speed in the target controller and reset it for smooth transition
        match mode {
            SpeedControlMode::Adaptive => {
                self.adaptive_controller.set_speed(current_speed);
                self.adaptive_controller.reset();

                // Set again after reset to maintain speed
                self.adaptive_controller.set_speed(current_speed);
            }
            SpeedControlMode::MinMax => {
                self.minmax_controller.set_speed(current_speed);
                self.minmax_controller.reset();

                // Set again after reset to maintain speed
                self.minmax_controller.set_speed(current_speed);
            }
        }
    }
}
