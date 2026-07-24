mod kind;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use kind::Kind;
use kind::kind_t;

pub mod error;

mod conversion;

mod property;
pub use property::PropertyRegistry;
pub use property::PropertyHandle;
pub use property::PropertyResolver;
pub use property::PropertyReader;
pub use property::PropertyReadHandle;
use qitech_framework_common::MachineIdentificationUnique;

pub mod config_property;
pub mod state_property;
pub mod measurement;
pub mod event;
pub mod command;

pub type JournalBuffer<T> = heapless::Vec<T, 1024>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: &'a str,
    postfix: &'a str,
}

#[derive(Debug, Default)]
pub struct Journal<E> {
    buffer: Rc<RefCell<JournalBuffer<E>>>,
}

impl<E> Journal<E> {
    pub fn append(&mut self, entry: E) -> Result<(), JournalAppendError> {
        self.buffer.borrow_mut().push(entry).map_err(|_| JournalAppendError)
    }

    pub fn iter(&mut self) {
        self.buffer
    }
}

#[derive(Debug)]
pub struct JournalHandle<T> {
    journal: Rc<RefCell<Journal<T>>>,
}

impl<T: Debug> JournalHandle<T> {
    pub(crate) fn new(journal: Rc<RefCell<Journal<T>>>) -> Self {
        Self { journal }
    }

    pub fn append(&self, entry: T) -> Result<(), JournalAppendError> {
        self.journal.borrow_mut().push(entry).map_err(|_| JournalAppendError)
    }
}

pub struct JournalAppendError;
