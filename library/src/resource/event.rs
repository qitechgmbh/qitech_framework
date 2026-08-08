use std::marker::PhantomData;

use chrono::Utc;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::CustomEventRecord;
use serde::Serialize;

use crate::resource::JournalHandle;
use crate::resource::LifetimeToken;
use crate::resource::LifetimeTokenProvider;
use crate::resource::journal::Journal;

pub type EventEmitResult = Result<(), EventEmitError>;
pub type EventEmitError = serde_json::Error;

pub struct EventEmitter<T: Serialize> {
    machine: MachineIdentificationUnique,
    path: &'static str,
    journal: JournalHandle<CustomEventRecord>,
    token: LifetimeToken,
    _marker: PhantomData<T>,
}

impl<T: Serialize> EventEmitter<T> {
    pub(crate) fn new(
        machine: MachineIdentificationUnique,
        path: &'static str,
        journal: JournalHandle<CustomEventRecord>,
        token: LifetimeToken,
    ) -> Self {
        Self {
            machine,
            path,
            journal,
            token,
            _marker: PhantomData,
        }
    }

    pub fn emit(&mut self, event: T) -> EventEmitResult {
        self.journal.append(CustomEventRecord {
            timestamp: Utc::now(),
            machine: self.machine,
            path: self.path.to_string(),
            data: serde_json::to_vec(&event)?,
        });

        assert!(!self.token.expired());
        Ok(())
    }
}

// --- registry ---
pub struct EventRegistry {
    machines: heapless::Vec<Entry, 64>,
}

impl EventRegistry {
    pub fn remove_machine(&mut self, ident: MachineIdentificationUnique) {
        self.machines.retain(|entry| entry.machine != ident);
    }
}

#[derive(Debug)]
struct Entry {
    machine: MachineIdentificationUnique,
    provider: LifetimeTokenProvider,
}

// --- registrar ---
pub struct EventRegistrar<'a> {
    registry: &'a mut EventRegistry,
    journal: &'a mut Journal<CustomEventRecord>,
    provider: LifetimeTokenProvider,
    machine: MachineIdentificationUnique,
    items: Vec<&'static str>,
}

impl<'a> EventRegistrar<'a> {
    pub(crate) fn new(
        registry: &'a mut EventRegistry,
        journal: &'a mut Journal<CustomEventRecord>,
        machine: MachineIdentificationUnique,
    ) -> Self {
        Self {
            registry,
            journal,
            provider: LifetimeTokenProvider::new(),
            machine,
            items: Default::default(),
        }
    }

    pub(crate) fn register<T: Serialize>(&mut self, path: &'static str) -> EventEmitter<T> {
        if self.items.contains(&path) {
            panic!("TODO: yield error");
        }

        EventEmitter {
            machine: self.machine,
            path,
            journal: self.journal.new_handle(),
            token: self.provider.new_token(),
            _marker: PhantomData,
        }
    }

    pub(crate) fn commit(self) {
        assert!(
            !self
                .registry
                .machines
                .iter()
                .any(|entry| entry.machine == self.machine)
        );

        self.registry
            .machines
            .push(Entry {
                machine: self.machine,
                provider: self.provider,
            })
            .expect("Exceeded max number of machines per runtime");
    }
}
