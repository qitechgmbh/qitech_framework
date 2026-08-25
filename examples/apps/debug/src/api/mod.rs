use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;

mod types;
use qitech_framework::MachineSchema;
use types::MachineInstance;

mod legacy;
pub use legacy::LegacySharedState;
pub use legacy::SocketIODispatcher;

mod server;
pub use server::Server;

#[derive(Default, Clone)]
pub struct Swappable<T: Clone>(Arc<ArcSwap<T>>);

impl<T: Clone> Swappable<T> {
    pub fn read(&self) -> arc_swap::Guard<Arc<T>> {
        self.0.load()
    }

    pub fn update<F>(&mut self, modify: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = (*self.0.load_full()).clone();
        modify(&mut value);
        self.0.store(Arc::new(value));
    }
}

#[derive(Default, Clone)]
pub struct SharedState {
    pub schemas: Swappable<HashMap<MachineIdentification, MachineSchema>>,
    pub machines: Swappable<HashMap<MachineInstanceIdentification, MachineInstance>>,
}
