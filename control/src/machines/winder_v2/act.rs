use std::time::Instant;

use qitech_framework::{MachineIdentificationUnique, machine::{Machine, SubscribeContext, error::{ActResult, SubscribeResult}}};
use qitech_lib::units::length::millimeter;
use super::WinderV2;
use super::LaserSubscription;

impl Machine for WinderV2 {
    fn act(&mut self) -> ActResult {
        let now = Instant::now();

        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        if let Some(laser) = self.laser_subscription.as_ref() {
            let last_speed = self.puller_speed_controller.last_speed;

            self.puller_speed_controller.adaptive.update_with_measurement(
                laser.current.get_as::<millimeter>(),
                laser.target.get_as::<millimeter>(),
                laser.lower.get_as::<millimeter>(),
                laser.upper.get_as::<millimeter>(),
                last_speed,
                Instant::now(),
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
