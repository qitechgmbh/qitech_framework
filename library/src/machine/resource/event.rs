use std::borrow::Cow;
use std::collections::HashSet;
use std::marker::PhantomData;

use chrono::Utc;
use qitech_framework_common::MachineEvent;
use qitech_framework_common::MachineIdentificationUnique;
use serde::Serialize;

use super::Journal;
use super::JournalHandle;
use super::Key;
use super::Kind;
use super::error::RegisterError;
use super::error::RegisterErrorKind;
use super::error::RegisterResult;

// -- machine handle ---
pub struct Emitter<T: Serialize> {
    source: MachineIdentificationUnique,
    path: &'static str,
    journal: JournalHandle<MachineEvent>,
    _marker: PhantomData<T>,
}

impl<T: Serialize> Emitter<T> {
    pub fn emit(&mut self, data: T) -> EventEmitResult {
        let event = MachineEvent {
            timestamp: Utc::now(),
            source: self.source,
            resource_path: Cow::Borrowed(self.path),
            data: serde_json::to_string(&data).map_err(EmitError::SerializeError)?,
        };

        self.journal
            .append(event)
            .map_err(|_| EmitError::JournalFull)
    }
}

// --- manager ---
pub struct Manager {
    registry: HashSet<Key<'static>>,
    journal: Journal<MachineEvent>,
}

impl Manager {
    pub(crate) fn unregister_machine(&mut self, ident: &MachineIdentificationUnique) {
        self.registry.retain(|key| &key.ident != ident);
    }
}

// --- registrar ---
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl Registrar<'_> {
    pub(crate) fn register<T>(&mut self, path: &'static str) -> RegisterResult<Emitter<T>>
    where
        T: Serialize,
    {
        let key = Key {
            ident: self.machine,
            path,
            postfix: "",
        };

        if !self.manager.registry.insert(key) {
            return Err(RegisterError {
                resource_kind: Kind::Event,
                resource_path: path,
                kind: RegisterErrorKind::Duplicate,
            });
        }

        Ok(Emitter {
            source: self.machine,
            path,
            journal: self.manager.journal.init_handle(),
            _marker: PhantomData,
        })
    }
}

// --- resolver ---
pub struct Resolver<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

// --- remote handle ---
pub struct RemoteHandle {}

// --- errors ---
pub type EventEmitResult = Result<(), EmitError>;

#[derive(Debug)]
pub(crate) enum EmitError {
    JournalFull,
    SerializeError(serde_json::Error),
}
