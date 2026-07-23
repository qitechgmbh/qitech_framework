use std::{borrow::Cow, cell::RefCell, collections::HashSet, marker::PhantomData, rc::Rc};
use chrono::Utc;
use serde::Serialize;
use qitech_framework_common::{MachineEvent, MachineIdentificationUnique};

use crate::machine::resource::{
    Journal, JournalHandle, Key,
    error::{RegisterError, RegisterErrorKind, RegisterResult},
    kind::Kind,
};

pub type EventEmitResult = serde_json::Result<()>;

pub struct Manager {
    registry: HashSet<Key<'static>>,
    journal: Rc<RefCell<Journal<MachineEvent>>>,
}

impl Manager {
    pub(crate) fn create<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
    ) -> RegisterResult<EventEmitter<T>> {
        let key = Key {
            ident,
            path,
            postfix: "",
        };

        if !self.registry.insert(key) {
            return Err(RegisterError {
                resource_kind: Kind::Event,
                resource_path: path,
                kind: RegisterErrorKind::AlreadyRegistered,
            });
        }

        let journal = JournalHandle::new(self.journal.clone());

        Ok(EventEmitter {
            source: ident,
            resource_path: Cow::Borrowed(path),
            journal,
            _marker: PhantomData,
        })
    }

    pub(crate) fn unregister_machine(&mut self) {
        // TODO: remove entries from registry
    }
}

pub struct EventEmitter<T> {
    source: MachineIdentificationUnique,
    resource_path: Cow<'static, str>,
    journal: JournalHandle<MachineEvent>,
    _marker: PhantomData<T>,
}

impl<T: Serialize> EventEmitter<T> {
    pub fn emit(&mut self, data: T) -> EventEmitResult {
        let data = serde_json::to_string(&data)?;

        self.journal.append(MachineEvent {
            source: self.source,
            resource_path: self.resource_path,
            data,
            timestamp: Utc::now(),
        });

        Ok(())
    }
}
