use std::fmt::Debug;
use chrono::Utc;
use control_core::{MachineEvent, MachineIdentificationUnique};
use serde::Serialize;
use crate::resource::WeakResourceJournal;

pub struct EventEmitter<T> {
    journal: WeakResourceJournal<MachineEvent>,
    ident: MachineIdentificationUnique,
    name: &'static str,
    _marker: PhantomData<T>,
    // rec_handle: MachineEventRecorderHandle<T>
}

/// journal
impl<T: Debug + Serialize> EventEmitter<T> {
    pub fn emit(&self, event: T) {

        let data = match serde_json::to_value(&event) {
            Ok(v) => v,
            Err(e) => {
                // self.log_handle.record(
                //     LogLevel::Error, 
                //     format!(
                //         "Failed to record event {} for machine {}: {e}",
                //         self.name, self.ident,
                //     ), 
                //     Default::default()
                // );
                return;
            }
        };

        let journal = self.journal.upgrade()
            .expect("Runtime must outlive journal handle");

        journal.borrow_mut().push(MachineEvent { 
            timestamp: Utc::now(), 
            ident: self.ident, 
            name: Cow::Borrowed(self.name), 
            data,
        });
    }
}