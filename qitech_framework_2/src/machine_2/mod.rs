use std::any::Any;
use std::time::Duration;

use qitech_framework_core::report::ActResult;

pub trait Machine: Any {
    const NAME: &'static str;

    /// Defines the update cycle of a machine.
    fn act(&mut self, dt: Duration) -> ActResult {
        _ = dt;
        Ok(())
    }
}
