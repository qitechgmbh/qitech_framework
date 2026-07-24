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

pub type EventEmitResult = Result<(), EmitError>;

pub struct Emitter<T: Serialize> {
    source: MachineIdentificationUnique,
    path: &'static str,
    journal: JournalHandle<MachineEvent>,
    _marker: PhantomData<T>,
}

impl<T: Serialize> Emitter<T> {
    pub fn emit(&mut self, data: T) -> EventEmitResult {
        self.journal
            .append(MachineEvent {
                timestamp: Utc::now(),
                source: self.source,
                resource_path: Cow::Borrowed(self.path),
                data: serde_json::to_string(&data).map_err(EmitError::SerializeError)?,
            })
            .map_err(|_| EmitError::JournalFull);

        Ok(())
    }
}

pub struct Manager {
    registry: HashSet<Key<'static>>,
    journal: Journal<MachineEvent>,
}

impl Manager {
    pub(crate) fn create<T: Serialize>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
    ) -> RegisterResult<Emitter<T>> {
        let key = Key {
            ident,
            path,
            postfix: "",
        };

        if !self.registry.insert(key) {
            return Err(RegisterError {
                resource_kind: Kind::Event,
                resource_path: path,
                kind: RegisterErrorKind::Duplicate,
            });
        }

        Ok(Emitter {
            source: ident,
            path,
            journal: self.journal.init_handle(),
            _marker: PhantomData,
        })
    }

    pub(crate) fn unregister_machine(&mut self, ident: &MachineIdentificationUnique) {
        self.registry.retain(|key| &key.ident != ident);
    }
}

#[derive(Debug)]
pub(crate) enum EmitError {
    JournalFull,
    SerializeError(serde_json::Error),
}
