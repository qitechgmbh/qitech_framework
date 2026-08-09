use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use qitech_framework_core::report::CommandRecord;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::CustomEventRecord;
use qitech_framework_core::report::StatePropertyRecord;

#[derive(Debug, Default)]
pub(crate) struct Journals {
    pub(crate) config_property: Journal<ConfigPropertyRecord>,
    pub(crate) state_property: Journal<StatePropertyRecord>,
    pub(crate) commands: Journal<CommandRecord>,
    pub(crate) events: Journal<CustomEventRecord>,
}

impl Journals {
    pub(crate) fn record_config(&mut self, value: ConfigPropertyRecord) {
        self.config_property.new_handle().append(value);
    }
}

#[derive(Debug)]
pub(crate) struct Journal<T> {
    buffer: Rc<RefCell<Vec<T>>>,
}

impl<T> Journal<T> {
    pub(crate) fn new_handle(&self) -> JournalHandle<T> {
        JournalHandle {
            buffer: self.buffer.clone(),
        }
    }

    pub(crate) fn drain_with(&mut self, mut f: impl FnMut(T)) {
        for entry in self.buffer.borrow_mut().drain(..) {
            f(entry);
        }
    }
}

impl<T> Default for Journal<T> {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct JournalHandle<T> {
    buffer: Rc<RefCell<Vec<T>>>,
}

impl<T: Debug> JournalHandle<T> {
    pub(crate) fn append(&self, entry: T) {
        self.buffer.borrow_mut().push(entry);
    }
}
