use std::time::Instant;

use control_runtime::{MachineIdentification, MachineIdentificationUnique};
use control_runtime::{Machine, MachineActResult};
use control_runtime::machine::{ConfigReaderHandle, MeasurementReaderHandle, ReactContext, StateProperty, SubscribeContext, SubscribeError, SubscribeResult};

mod types;
use qitech_lib::units::length::millimeter;
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

use crate::machines::LaserV1;

struct LaserSubscription {
    ident: MachineIdentificationUnique,
    current: MeasurementReaderHandle<millimeter>,
    target: ConfigReaderHandle<millimeter>,
    lower: ConfigReaderHandle<millimeter>,
    upper: ConfigReaderHandle<millimeter>,
}

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

    // --- connection ---
    laser_subscription: Option<LaserSubscription>
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

    fn react(&mut self, ctx: &ReactContext) -> MachineActResult {
        let Some(handles) = &self.laser_subscription else {
            return Ok(());
        };
        
        self.puller.update_with_laser_data(
            Instant::now(), 
            ctx.measurements.read(&handles.current)?, 
            *ctx.config.read(&handles.target)?, 
            *ctx.config.read(&handles.upper)?, 
            *ctx.config.read(&handles.lower)?
        );
    
        Ok(())
    }

    fn subscribe(&mut self, ctx: &SubscribeContext) -> SubscribeResult {
        if MachineIdentification::from(ctx.ident) != LaserV1::IDENTIFICATION {
            // not a laser_v1
            return Err(SubscribeError::UnsupportedMachine);
        }

        if self.laser_subscription.is_some() {
            // already has a connection
            return Err(SubscribeError::TooManySubscriptions);
        }

        self.laser_subscription = Some(LaserSubscription { 
            ident: ctx.ident, 
            current: ctx.measurements.resolve("diameter")?,
            target: ctx.config.resolve("diameter.target")?,
            upper: ctx.config.resolve("diameter.tolerance.upper")?,
            lower: ctx.config.resolve("diameter.tolerance.lower")?,
        });

        Ok(())
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        if let Some(sub) = &mut self.laser_subscription && sub.ident == ident {
            self.laser_subscription = None;
        }
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
        // self.travserse.goto_limit_inner();
    }

    pub fn traverse_goto_limit_outer(&mut self) {
        // self.travserse.goto_limit_outer();
    }

    pub fn traverse_goto_home(&mut self) {
        // self.travserse.goto_home();
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