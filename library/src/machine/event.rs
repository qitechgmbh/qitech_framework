use std::marker::PhantomData;

use serde::Serialize;

use crate::resource::JournalHandle;

pub struct EventEmitter<T: Serialize> {
    pub(super) journal: JournalHandle<String>,
    pub(super) _marker: PhantomData<T>,
}

impl<T: Serialize> EventEmitter<T> {
    pub fn emit(&mut self, event: &T) {
        let data = serde_json::to_string(event).unwrap();
        self.journal.record(data);
    }
}
