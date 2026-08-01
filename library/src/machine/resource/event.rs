use std::any::TypeId;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::rc::Rc;
use std::rc::Weak;

use chrono::Utc;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::MachineEmittedEvent;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::Journal;
use super::JournalHandle;
use super::Key;
use super::error::RegisterError;
use super::error::RegisterResult;
use crate::machine::resource::subscription::SubscribeError;
use crate::machine::resource::subscription::SubscriptionRegistry;
use crate::machine::resource::subscription::SubscriptionToken;

pub struct Emitter<T: Serialize> {
    machine: MachineIdentificationUnique,
    path: &'static str,
    slot: Weak<RefCell<Slot>>,
    journal: JournalHandle<MachineEmittedEvent>,
    _marker: PhantomData<T>,
}

impl<T: Serialize> Emitter<T> {
    pub fn emit(&mut self, event: T) -> EventEmitResult {
        self.journal.append(MachineEmittedEvent {
            timestamp: Utc::now(),
            machine: self.machine,
            path: Cow::Borrowed(self.path),
            data: serde_json::to_string(&event)?,
        });

        let slot = self.slot.upgrade().expect("must not outlive manager entry");
        let mut slot = slot.borrow_mut();

        if slot.subscriber_count > 0 {
            let data = postcard::to_allocvec(&event).expect("json succeeded");
            slot.cache.push(data);
        }

        Ok(())
    }
}

// --- manager ---
pub struct Manager {
    registry: HashMap<Key<'static>, Entry>,

    /// journal of all emitted events
    journal: Journal<MachineEmittedEvent>,

    /// list of entries that have subscribers
    subscribed: Vec<Key<'static>>,

    /// all active subscriptions
    subscriptions: SubscriptionRegistry,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            registry: Default::default(),
            journal: Journal::default(),
            subscribed: Default::default(),
            subscriptions: Default::default(),
        }
    }

    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
    ) -> RegisterResult<Emitter<T>>
    where
        T: Serialize + 'static,
    {
        let key = Key::from_str(ident, path);

        if self.registry.contains_key(&key) {
            return Err(RegisterError::Duplicate);
        }

        let slot = Rc::new(RefCell::new(Slot {
            subscriber_count: 0,
            cache: Default::default(),
        }));

        let emitter = Emitter {
            machine: ident,
            path,
            slot: Rc::downgrade(&slot),
            journal: self.journal.new_handle(),
            _marker: PhantomData,
        };

        let entry = Entry {
            type_id: TypeId::of::<T>(),
            slot,
            cache: Default::default(),
            subscribers: 0,
        };

        self.registry.insert(key, entry);
        Ok(emitter)
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.retain(|k, _| k.ident != ident);
        self.subscribed.retain(|x| x.ident != ident);
        self.subscriptions.unregister_producer(ident);
    }

    pub fn create_subscriber<T: DeserializeOwned + 'static>(
        &mut self,
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
        resource: &'static str,
    ) -> Result<SubscribedEvent<T>, SubscribeError> {
        let key = Key::from_str(producer, resource);

        let (slot, data) = {
            let Some(entry) = self.registry.get(&key) else {
                return Err(SubscribeError::NoSuchProperty);
            };

            // ensure the user provided the correct Type
            if entry.type_id != TypeId::of::<T>() {
                return Err(SubscribeError::InvalidType);
            }

            (entry.slot.clone(), &entry.cache)
        };

        let token = self.subscriptions.register(producer, consumer, resource)?;

        {
            let mut slot = slot.borrow_mut();
            slot.subscriber_count += 1;

            if slot.subscriber_count == 1 {
                self.subscribed.push(key);
            }
        }

        let subscribed_event = SubscribedEvent {
            token,
            data: Rc::downgrade(data),
            _marker: PhantomData,
        };

        Ok(subscribed_event)
    }

    pub fn remove_subscription(
        &mut self,
        producer: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) {
        for resource in self.subscriptions.subscribed_resources(producer, consumer) {
            let key = Key {
                ident: producer,
                path: Cow::Owned(resource.to_string()),
            };

            let entry = self
                .registry
                .get_mut(&key)
                .expect("created subscriber for non existing item");

            entry.subscribers = entry.subscribers.saturating_sub(1);

            if entry.subscribers == 0 {
                self.subscribed.retain(|v| *v != key);
                entry.cache.borrow_mut().clear();
            }
        }

        self.subscriptions.unregister(producer, consumer);
    }

    pub fn sync_cache(&mut self) {
        for key in self.subscribed.clone() {
            let entry = self.registry.get_mut(&key).expect("Must be in sync");

            // extract the cached entries out of slot
            let cache = {
                let mut slot = entry.slot.borrow_mut();
                mem::take(&mut slot.cache)
            };

            // replace subscriber-visible cache
            entry.cache.replace(cache);
        }
    }

    pub fn drain_journal(&mut self, f: impl FnMut(MachineEmittedEvent)) {
        self.journal.drain_with(f);
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

// --- types ---
type EventCache = Vec<Vec<u8>>;

struct Entry {
    type_id: TypeId,
    slot: Rc<RefCell<Slot>>,
    cache: Rc<RefCell<EventCache>>,
    subscribers: u32,
}

struct Slot {
    subscriber_count: u32,
    cache: EventCache,
}

pub struct SubscribedEvent<T: DeserializeOwned> {
    token: Weak<SubscriptionToken>,
    data: Weak<RefCell<Vec<Vec<u8>>>>,
    _marker: PhantomData<T>,
}

impl<T> SubscribedEvent<T>
where
    T: DeserializeOwned,
{
    pub fn for_each<F>(&self, mut f: F) -> Result<(), postcard::Error>
    where
        F: FnMut(T),
    {
        self.token.upgrade().expect("Token outlived subscription");
        let data = self.data.upgrade().expect("token is valid");
        let data = data.borrow();

        for bytes in data.iter() {
            let value = postcard::from_bytes::<T>(bytes)?;
            f(value);
        }

        Ok(())
    }
}

// --- errors ---
pub type EventEmitResult = Result<(), serde_json::Error>;

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_core::ident::MachineIdentification;
    use serde::Deserialize;

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

        let mut mgr = Manager::new();
        // --- simple ---
        #[derive(Serialize, Deserialize)]
        struct SimpleEvent {
            a: i64,
            b: f64,
            c: i32,
        }

        let mut emitter = mgr.register::<SimpleEvent>(ident, "simple")?;
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

        let mut emitter = mgr.register::<MediumEvent>(ident, "medium")?;

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
