use crate::SharedState;
use crate::modules::DatabaseModule;

pub struct ClickhouseModule {
    state: SharedState,
}

impl DatabaseModule for ClickhouseModule {
    fn init() -> Self {
        todo!()
    }

    async fn start() {
        todo!()
    }
}
