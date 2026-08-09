use std::marker::PhantomData;

use chrono::Utc;
use qitech_framework_core::report::EventEmitterRecord;
use serde::Serialize;

use crate::resource::JournalHandle;
use crate::resource::ResourceKey;

pub struct EventEmitter<T: Serialize> {
    pub(super) key: ResourceKey,
    pub(super) journal: JournalHandle<EventEmitterRecord>,
    pub(super) _marker: PhantomData<T>,
}

impl<T: Serialize> EventEmitter<T> {
    pub fn emit(&mut self, event: &T) {
        let data = serde_json::to_string(event).unwrap();

        self.journal.append(EventEmitterRecord {
            timestamp: Utc::now(),
            machine: self.key.ident,
            path: self.key.path.to_string(),
            data,
        });
    }
}
