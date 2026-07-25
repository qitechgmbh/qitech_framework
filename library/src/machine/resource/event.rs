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
            data: serde_json::to_string(&data)?,
        };

        self.journal.append(event);
        Ok(())
    }
}

// --- manager ---
pub struct Manager {
    registry: HashSet<Key<'static>>,
    journal: Journal<MachineEvent>,
}

impl Manager {
    pub(crate) fn new() -> Self {
        Self {
            registry: Default::default(),
            journal: Journal::new(),
        }
    }

    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
    ) -> RegisterResult<Emitter<T>>
    where
        T: Serialize,
    {
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
            journal: self.journal.new_handle(),
            _marker: PhantomData,
        })
    }

    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.retain(|key| key.ident != ident);
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
pub type EventEmitResult = Result<(), serde_json::Error>;

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_common::MachineIdentification;

    use super::*;

    #[test]
    pub fn register_and_use() -> anyhow::Result<()> {
        let ident = MachineIdentificationUnique {
            identification: MachineIdentification {
                vendor_id: 0,
                machine_id: 0,
            },
            serial: 0,
        };

        let mut r = Manager::new();

        // --- simple ---
        #[derive(Serialize)]
        struct SimpleEvent {
            a: i64,
            b: f64,
            c: i32,
        }

        let mut emitter = r.register::<SimpleEvent>(ident, "simple")?;
        emitter.emit(SimpleEvent {
            a: 0,
            b: 1.0,
            c: 100,
        })?;

        // --- complex ---
        #[derive(Serialize)]
        #[allow(unused)]
        enum SensorStatus {
            Ok,
            Warning,
            Error,
        }

        #[derive(Serialize)]
        struct Position {
            x: f64,
            y: f64,
            z: f64,
        }

        #[derive(Serialize)]
        struct Motor {
            id: u8,
            enabled: bool,
            temperature: Option<f32>,
        }

        #[derive(Serialize)]
        struct MediumEvent {
            machine_name: String,
            status: SensorStatus,
            position: Position,
            motors: Vec<Motor>,
            tags: std::collections::BTreeMap<String, String>,
        }

        let mut emitter = r.register::<MediumEvent>("medium")?;

        let mut tags = std::collections::BTreeMap::new();
        tags.insert("recipe".into(), "part_a".into());
        tags.insert("operator".into(), "alice".into());

        emitter.emit(MediumEvent {
            machine_name: "Assembly Cell 1".into(),
            status: SensorStatus::Warning,
            position: Position {
                x: 1.2,
                y: -3.4,
                z: 5.6,
            },
            motors: vec![
                Motor {
                    id: 1,
                    enabled: true,
                    temperature: Some(42.5),
                },
                Motor {
                    id: 2,
                    enabled: false,
                    temperature: None,
                },
            ],
            tags,
        })?;

        // TODO: assert correct data
        mgr.journal.drain_with(|event| {
            println!("data: {:?}", event);
        });

        Ok(())
    }
}
