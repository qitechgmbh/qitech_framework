use std::time::{Duration, Instant};
use control_runtime::{Machine, MachineActResult, StateProperty} ;

mod tension_arm;
use tension_arm::TensionArm;

mod build;

pub struct WinderV1 {
    // --- state ---
    pub mode: StateProperty<Mode>,

    // --- devices ---
    tension_arm: TensionArm,

    counter: Instant,
}

impl Machine for WinderV1 {
    fn act(&mut self) -> MachineActResult {
        
        let now = Instant::now();
            if now.duration_since(self.counter) > Duration::from_secs(1) {
            let angle_deg = self.tension_arm.get_angle().unwrap().get::<qitech_lib::units::angle::degree>();

            // Wrap [270;<360] to [-90; 0]
            // This is done to reduce flicker in the graphs around the zero point
            let angle_deg = if angle_deg >= 270.0 {
                angle_deg - 360.0
            } else {
                angle_deg
            };

            println!("angle: {:?}", angle_deg);

            self.counter = now;
        }


        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Standby,
    Hold,
    Pull,
    Wind,
}
