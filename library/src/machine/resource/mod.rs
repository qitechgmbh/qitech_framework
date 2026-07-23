mod kind;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use kind::Kind;
use kind::kind_t;

pub mod error;

mod conversion;
use conversion::Convertible;
use conversion::BoundedMeta;
use conversion::PropertyMeta;

mod property;
pub use property::PropertyRegistry;
pub use property::PropertyHandle;
pub use property::PropertyResolver;
pub use property::PropertyReader;
pub use property::PropertyAccessHandle;
use qitech_framework_common::MachineIdentificationUnique;

pub mod config_property;
pub mod state_property;
pub mod measurement;
pub mod event;
pub mod command;

pub type Journal<T> = heapless::Vec<T, 1024>;

#[derive(Debug)]
pub struct JournalHandle<T> {
    journal: Rc<RefCell<Journal<T>>>,
}

impl<T: Debug> JournalHandle<T> {
    pub(crate) fn new(journal: Rc<RefCell<Journal<T>>>) -> Self {
        Self { journal }
    }

    pub fn append(&self, entry: T) {
        self.journal.borrow_mut().push(entry)
            .expect("Should never reach this capacity");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: &'a str,
    postfix: &'a str,
}