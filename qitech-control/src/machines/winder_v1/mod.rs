use std::time::Instant;
use control_runtime::{Machine, MachineActResult, StateProperty} ;

mod tension_arm;
use tension_arm::TensionArm;

mod traverse;
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
