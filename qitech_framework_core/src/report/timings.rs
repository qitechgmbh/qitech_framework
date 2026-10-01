use std::mem;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingsReport {
    // total number of cycles
    cycle_count: u32,

    /// total duration spent executing
    duration_total: Duration,

    /// duration of the longest cycle
    duration_peak: Duration,

    // cycles that exceeded cycle_timeout
    overrun_count: u32,
}

impl TimingsReport {
    pub fn reset(&mut self) {
        self.cycle_count = 0;
        self.duration_total = Duration::ZERO;
        self.duration_peak = Duration::ZERO;
        self.overrun_count = 0;
    }

    pub fn record(&mut self, duration: Duration, budget: Duration) {
        self.cycle_count += 1;
        self.duration_total += duration;
        if duration > self.duration_peak {
            self.duration_peak = duration;
        }

        if duration > budget {
            self.overrun_count += 1;
        }
    }

    pub fn duration_avg(&self) -> Duration {
        if self.cycle_count == 0 {
            Duration::ZERO
        } else {
            self.duration_peak / self.cycle_count
        }
    }

    /// Take the current stats and reset for the next export window.
    pub fn take(&mut self) -> TimingsReport {
        mem::take(self)
    }
}
