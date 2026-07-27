use std::time::Instant;

mod types;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::machine::Machine;
use qitech_framework::machine::MachineInterface;
use qitech_framework::machine::SubscribeContext;
use qitech_framework::machine::SubscribedProperty;
use qitech_framework::machine::error::ActResult;
use qitech_framework::machine::error::SubscribeResult;
use qitech_framework::machine::resource::StateProperty;
use qitech_lib::units::Length;
use types::Mode;

mod build;
mod utils;

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

mod commands;
use commands::Commands;

struct LaserSubscription {
    ident: MachineIdentificationUnique,
    current: SubscribedProperty<Length>,
    target: SubscribedProperty<Length>,
    lower: SubscribedProperty<Length>,
    upper: SubscribedProperty<Length>,
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
    laser_subscription: Option<LaserSubscription>,
}

// load_schema!("winder_v1.yaml");
impl MachineInterface for WinderV1 {
    const SCHEMA: &'static str = include_str!("../../../schemas/winder_v1.yaml");
}

impl Machine for WinderV1 {
    fn act(&mut self) -> ActResult {
        let now = Instant::now();
        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);
        self.travserse.update(&self.spool);

        match self.spool_target.evaluate(now, &self.puller) {
            SpoolTargetReachedAction::None => {}
            SpoolTargetReachedAction::Pull => {

                // TODO: invoke state change
            }
            SpoolTargetReachedAction::Hold => {
                // TODO: invoke state change
            }
        }

        if let Some(laser) = self.laser_subscription.as_ref() {
            self.puller.update_with_laser_data(
                Instant::now(),
                laser.current.get(),
                laser.target.get(),
                laser.upper.get(),
                laser.lower.get(),
            );
        }

        Ok(())
    }

    fn subscribe(&mut self, mut ctx: SubscribeContext) -> SubscribeResult<()> {
        self.laser_subscription = Some(LaserSubscription {
            ident: ctx.producer(),
            current: ctx.subscribe_measurement("diameter")?,
            target: ctx.subscribe_config("diameter.target")?,
            upper: ctx.subscribe_config("diameter.tolerance.upper")?,
            lower: ctx.subscribe_config("diameter.tolerance.lower")?,
        });

        Ok(())
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        if let Some(sub) = &mut self.laser_subscription
            && sub.ident == ident
        {
            self.laser_subscription = None;
        }
    }
}
