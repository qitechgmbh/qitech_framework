use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use chrono::Utc;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::EventRecord;
use qitech_framework_core::report::StatePropertyEvent;

use crate::resource::ResourceKey;

#[derive(Debug, Default)]
pub(crate) struct Journals {
    pub(crate) config_property: Journal<ConfigPropertyEvent>,
    pub(crate) state_property: Journal<StatePropertyEvent>,
    pub(crate) command: Journal<CommandEvent>,
    pub(crate) event: Journal<String>,
}

#[derive(Debug)]
pub(crate) struct Journal<T: Debug> {
    buffer: Rc<RefCell<Vec<EventRecord<T>>>>,
}

impl<T: Debug> Journal<T> {
    pub(crate) fn record(&self, machine: MachineIdentificationUnique, path: &str, event: T) {
        self.buffer.borrow_mut().push(EventRecord {
            timestamp: Utc::now(),
            machine,
            path: path.to_string(),
            event,
        });
    }

    pub(crate) fn new_handle(&self, key: ResourceKey) -> JournalHandle<T> {
        JournalHandle {
            buffer: self.buffer.clone(),
            key,
        }
    }

    pub(crate) fn drain_with(&mut self, mut f: impl FnMut(EventRecord<T>)) {
        for entry in self.buffer.borrow_mut().drain(..) {
            f(entry);
        }
    }

    pub(crate) fn import(&mut self, other: Journal<T>) {
        let mut records = self.buffer.borrow_mut();
        records.extend(other.buffer.borrow_mut().drain(..));
    }
}

impl<T: Debug> Default for Journal<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct JournalHandle<T> {
    buffer: Rc<RefCell<Vec<EventRecord<T>>>>,
    key: ResourceKey,
}

impl<T: Debug> JournalHandle<T> {
    pub(crate) fn record(&self, event: T) {
        self.buffer.borrow_mut().push(EventRecord {
            timestamp: Utc::now(),
            machine: self.key.ident,
            path: self.key.path.to_string(),
            event,
        });
    }
}
