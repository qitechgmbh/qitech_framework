use std::{cell::RefCell, fmt::{self, Debug}, rc::Rc};

use control_core::MachineResourceKind;

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

#[derive(Debug)]
pub struct RegisterError {
    pub resource_kind: MachineResourceKind,
    pub resource_name: &'static str,
    pub reason: RegisterErrorReason,
}

#[derive(Debug)]
pub enum RegisterErrorReason {
    AlreadyRegistered,
    RegistryFull,
    NameTooLarge,
}

#[derive(Debug, Clone, Copy)]
pub enum ResolveError {
    NoSuchProperty,
    InvalidType,
}

#[derive(Debug)]
pub struct ReadError;

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expired handle")
    }
}

impl std::error::Error for ReadError {}