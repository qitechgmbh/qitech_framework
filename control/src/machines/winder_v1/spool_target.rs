use std::time::Instant;

use qitech_lib::units::{ConstZero, Length, length::meter, velocity::meter_per_second};

use crate::machines::winder_v1::puller::Puller;

#[derive(Debug, Clone, Copy, Default)]
pub enum SpoolTargetReachedAction {
    #[default]
    None,
    Pull,
    Hold,
}

#[derive(Debug)]
pub struct SpoolTarget {
    action: SpoolTargetReachedAction,
    target: Length,
    progress: Length,
    last_check: Instant,
}

impl SpoolTarget {
    pub fn evaluate(&mut self, now: Instant, puller: &Puller) -> SpoolTargetReachedAction {
        let dt = now
            .duration_since(self.last_check)
            .as_secs_f64();

        let meters_pulled_this_interval = Length::new::<meter>(
            puller.speed().get::<meter_per_second>() * dt,
        );

        self.progress += meters_pulled_this_interval;

        if self.progress >= self.target {
            self.reset(now);
            self.action
        } else {
            self.last_check = now;
            SpoolTargetReachedAction::None
        }
    }

    pub fn reset(&mut self, now: Instant) {
        self.progress = Length::ZERO;
        self.last_check = now;
    }
}
