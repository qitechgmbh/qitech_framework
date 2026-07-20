use std::collections::HashMap;
use chrono::Utc;
use control_core::{LogLevel, LogOrigin, LogRecord};
use crate::resource::WeakResourceJournal;

#[derive(Debug)]
pub struct LogWriter {
    weak: WeakResourceJournal<LogRecord>,
    origin: LogOrigin,
}

impl LogWriter {
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
