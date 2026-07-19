use std::{borrow::Cow, cell::RefCell, collections::HashMap, fmt::Debug, marker::PhantomData, rc::{Rc, Weak}};
use serde::Serialize;
use chrono::Utc;
use control_core::{LogLevel, LogOrigin, LogRecord, MachineConfigMutation, MachineEvent, MachineIdentificationUnique, MachineStateMutation, OperationResult, Origin, ScalarValue};

type RecordLog<T> = Rc<RefCell<Vec<T>>>;
type WeakRecordLog<T> = Weak<RefCell<Vec<T>>>;

#[derive(Debug)]
pub struct DataRecorder {
    configs: RecordLog<MachineConfigMutation>,
    states: RecordLog<MachineStateMutation>,
    events: RecordLog<MachineEvent>,
    logs: RecordLog<LogRecord>,
}

impl DataRecorder {
    pub(crate) fn new() -> Self {
        Self { 
            configs: Default::default(), 
            states: Default::default(), 
            events: Default::default(), 
            logs: Default::default(), 
        }
    }

    pub(crate) fn create_config_handle(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> ConfigRecorderHandle {
        ConfigRecorderHandle {
            weak: Rc::downgrade(&self.configs),
            ident,
            name,
        }
    }

    pub(crate) fn create_state_handle(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> StateRecorderHandle {
        StateRecorderHandle {
            weak: Rc::downgrade(&self.states),
            ident,
            name,
        }
    }

    pub(crate) fn create_event_handle<T: Serialize>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> MachineEventRecorderHandle<T> {
        // create log handle for reporting problems
        let log_handle = self.create_log_handle(LogOrigin::Machine(ident));

        MachineEventRecorderHandle {
            weak: Rc::downgrade(&self.events),
            log_handle,
            ident,
            name,
            _marker: PhantomData,
        }
    }

    pub(crate) fn create_log_handle(
        &mut self,
        origin: LogOrigin,
    ) -> LogRecorderHandle {
        LogRecorderHandle {
            weak: Rc::downgrade(&self.logs),
            origin,
        }
    }
}

#[derive(Debug)]
pub struct ConfigRecorderHandle {
    weak: WeakRecordLog<MachineConfigMutation>,
    ident: MachineIdentificationUnique,
    name: &'static str,
}

impl ConfigRecorderHandle {
    pub fn record(
        &mut self,
        origin: Origin,
        value: ScalarValue,
        result: OperationResult,
    ) {
        let log = self
            .weak
            .upgrade()
            .expect("Recorder dropped while handle still alive");

        log.borrow_mut().push(MachineConfigMutation {
            timestamp: Utc::now(),
            ident: self.ident,
            name: Cow::Borrowed(self.name),
            value,
            origin,
            result,
        });
    }
}

#[derive(Debug)]
pub struct StateRecorderHandle {
    weak: WeakRecordLog<MachineStateMutation>,
    ident: MachineIdentificationUnique,
    name: &'static str,
}

impl StateRecorderHandle {
    pub fn record(&mut self, value: ScalarValue) {
        let log = self
            .weak
            .upgrade()
            .expect("Recorder dropped while handle still alive");

        log.borrow_mut().push(MachineStateMutation {
            timestamp: Utc::now(),
            ident: self.ident,
            name: Cow::Borrowed(self.name),
            value,
        });
    }
}

#[derive(Debug)]
pub struct MachineEventRecorderHandle<T: Serialize> {
    weak: WeakRecordLog<MachineEvent>,
    log_handle: LogRecorderHandle,
    ident: MachineIdentificationUnique,
    name: &'static str,
    _marker: PhantomData<T>
}

impl<T: Debug + Serialize> MachineEventRecorderHandle<T> {
    pub fn record(&mut self, event: &T) {
        let log = self
            .weak
            .upgrade()
            .expect("Recorder dropped while handle still alive");

        let data = match postcard::to_allocvec(&event) {
            Ok(v) => v,
            Err(e) => {
                self.log_handle.record(
                    LogLevel::Error, 
                    format!(
                        "Failed to record event {} for machine {}: {e}",
                        self.name, self.ident,
                    ), 
                    Default::default()
                );
                return;
            }
        };

        // TODOD: 
        // log.borrow_mut().push(MachineEvent { 
        //     timestamp: Utc::now(), 
        //     ident: self.ident, 
        //     name: Cow::Borrowed(self.name), 
        //     data,
        // });
    }
}

#[derive(Debug)]
pub struct LogRecorderHandle {
    weak: WeakRecordLog<LogRecord>,
    origin: LogOrigin,
}

impl LogRecorderHandle {
    pub fn record(&mut self, level: LogLevel, message: String, attributes: HashMap<String, String>) {
        let log = self
            .weak
            .upgrade()
            .expect("Recorder dropped while handle still alive");

        log.borrow_mut().push(LogRecord { 
            timestamp: Utc::now(), 
            level,
            origin: self.origin, 
            message,
            attributes,
        }); 
    }
}
