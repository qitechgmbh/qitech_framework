use qitech_framework_common::{RuntimeReport, RuntimeRequest};

pub trait Bridge {
    fn init() -> Self;

    /// Drains up to `max` currently-buffered requests without blocking.
    fn get_requests(&mut self, max: usize) -> impl Iterator<Item = RuntimeRequest> + '_;


    fn export(&mut self, data: &RuntimeReport);
}

