use std::{cell::RefCell, fmt::Debug, rc::{Rc, Weak}};
use control_core::{LogRecord, MachineConfigMutation, MachineEvent, MachineStateMutation, };

pub type Journal<T> = Rc<RefCell<Vec<T>>>;
pub type WeakJournal<T> = Weak<RefCell<Vec<T>>>;

#[derive(Debug)]
pub struct Journals {
    configs: Journal<MachineConfigMutation>,
    states: Journal<MachineStateMutation>,
    events: Journal<MachineEvent>,
    logs: Journal<LogRecord>,
}

/*
impl Recorders {
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
    weak: WeakRecordLogs<MachineConfigMutation>,
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
    weak: WeakRecordLogs<MachineStateMutation>,
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
pub struct MachineEventRecorderHandle<T> {
    weak: WeakRecordLogs<MachineEvent>,
    log_handle: LogRecorderHandle,
    ident: MachineIdentificationUnique,
    name: &'static str,
    _marker: PhantomData<T>
}

impl<T: Debug + Serialize> MachineEventRecorderHandle<T> {
    pub fn record(&self, event: T) {
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
    weak: WeakRecordLogs<LogRecord>,
    origin: LogOrigin,
}

impl LogRecorderHandle {
    pub fn record(&self, level: LogLevel, message: String, attributes: HashMap<String, String>) {
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
*/