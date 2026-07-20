use std::{cell::RefCell, fmt, rc::Rc};

pub type Journal<T> = heapless::Vec<T, 1024>;

#[derive(Debug)]
pub struct JournalHandle<T> {
    journal: Rc<RefCell<Journal<T>>>,
}

impl<T> JournalHandle<T> {
    pub(crate) fn new(journal: Rc<RefCell<Journal<T>>>) -> Self {
        Self { journal }
    }

    pub fn append(&self, entry: T) {
        self.journal.borrow_mut().push(entry);
    }
}

pub enum RegisterError {
    NameTooLarge { 
        name: String,
    },
    NameRegistryFull {
        name: String 
    },
    RegistryFull { 
        name: &'static str 
    },
    AlreadyRegistered { 
        name: &'static str 
    },
    TypeTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
    AlignmentTooLarge { 
        r#type: &'static str, 
        name: &'static str 
    },
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