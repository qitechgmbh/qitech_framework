use std::marker::PhantomData;

use chrono::Utc;
use qitech_framework_core::report::CustomEventRecord;
use serde::Serialize;

use crate::resource::JournalHandle;
use crate::resource::ResourceKey;

pub struct EventEmitter<T: Serialize> {
    pub(super) key: ResourceKey,
    pub(super) journal: JournalHandle<CustomEventRecord>,
    pub(super) _marker: PhantomData<T>,
}

impl<T: Serialize> EventEmitter<T> {
    pub fn emit(&mut self, event: &T) {
        panic!("Event emitted");

        let data = serde_json::to_vec(event).unwrap();

        self.journal.append(CustomEventRecord {
            timestamp: Utc::now(),
            machine: self.key.ident,
            path: self.key.path.to_string(),
            data,
        });
    }
}
