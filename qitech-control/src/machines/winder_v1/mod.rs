use std::time::Instant;

use control_runtime::DataRegistry;
use control_runtime::{Machine, MachineActResult, StateProperty} ;

mod types;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use types::Mode;
use types::Commands;

mod utils;
mod build;

mod tension_arm;
use tension_arm::TensionArm;

mod spool;
use spool::Spool;

mod puller;
use puller::Puller;

mod traverse;
use traverse::Traverse;

mod spool_target;
use spool_target::SpoolTarget;
use spool_target::SpoolTargetReachedAction;

pub struct WinderV1 {
    // --- state ---
    mode: StateProperty<Mode>,

    // --- commands ---
    commands: Commands,

    // --- virtual devices / sub systems ---
    spool: Spool,
    puller: Puller,
    travserse: Traverse,
    tension_arm: TensionArm,

    // --- sub systems ---
    spool_target: SpoolTarget,
}

impl Machine for WinderV1 {
    fn act(&mut self) -> MachineActResult {
        let now = Instant::now();
        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);
        self.travserse.update(&self.spool);

        match self.spool_target.evaluate(now, &self.puller) {
            SpoolTargetReachedAction::None => {},
            SpoolTargetReachedAction::Pull => {

                // TODO: invoke state change
            },
            SpoolTargetReachedAction::Hold => {
                // TODO: invoke state change
            },
        }

        Ok(())
    }

    fn react(&mut self, registry: &DataRegistry) -> MachineActResult {
        let reg = registry.of_machine(laser_ident);
        let current = reg.measurements.get("diameter")?;
        let target  = reg.config.get("diameter.target")?;
        let upper   = reg.config.get("diameter.tolerance.upper")?;
        let lower   = reg.config.get("diameter.tolerance.lower")?;
    
        let last_speed = self.puller_speed_controller.last_speed;
    
        self.puller_speed_controller
            .adaptive
            .update_with_measurement(
                current,
                target,
                lower,
                upper,
                last_speed,
                Instant::now(),
            );

        Ok(())
    }

    fn attach(&mut self, ctx: AttachmentContext) {
        ctx.
    }
}

// --- traverse utilities ---
impl WinderV1 {

}

pub struct EnterStandyCommand {

}

impl EnterStandyCommand {
    pub fn execute(machine: &mut WinderV1) {

    }
}


// --- commands ---
impl WinderV1 {
    pub fn enter_standby(&mut self) {

    }

    pub fn enter_hold(&mut self) {
        
    }

    pub fn start_pulling(&mut self) {
        
    }

    pub fn start_winding(&mut self) {
        
    }

    pub fn enable_laser(&mut self) {
        // self.travserse.enable_laser();
    }

    pub fn disable_laser(&mut self) {
        // self.travserse.disable_laser();
    }

    pub fn traverse_goto_limit_inner(&mut self) {
        self.travserse.goto_limit_inner();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        self.travserse.goto_limit_outer();
    }

    pub fn traverse_goto_home(&mut self) {
        self.travserse.goto_home();
    }
}

pub struct EnterStandbyCommand;
impl MachineCommand<WinderV1> for EnterStandbyCommand {
    type Args = ();

    fn execute(machine: &mut WinderV1, args: Self::Args) -> MachineCommandResult {
        machine.puller.update(Instant::now());
        Ok(())
    }
}


trait MachineCommand<M> {
    type Args: DeserializeOwned;
    fn execute(machine: &mut M, args: Self::Args) -> MachineCommandResult;
}

type MachineCommandResult = Result<(), MachineCommandExeucteError>;

pub enum MachineCommandExeucteError {
    OutOfBounds,
    InvalidInput,
    MachineError,
    Custom(),
}