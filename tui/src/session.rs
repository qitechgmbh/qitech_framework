use std::collections::HashMap;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use qitech_framework::MachineIdentification;
use qitech_framework::session::ControllerTransport;
use qitech_framework::session::controller::SessionHandshake;
use qitech_framework::session::error::SchemaSyncError;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::schema::MachineSchema;

use crate::types::AppAction;

pub enum SessionMessage {
    Schemas(HashMap<MachineIdentification, MachineSchema>),
    InitEvent(RuntimeInitEvent),
    Finished,
    Running,
    Report(Box<RuntimeReport>),
    Disconnected,
}

pub fn run<T: ControllerTransport>(
    session: SessionHandshake<T>, 
    tx: Sender<SessionMessage>,
    rx: Receiver<AppAction>,
) {
    if wrapped_run(session, &tx, rx).is_err() {
        tx.send(SessionMessage::Disconnected)
            .expect("should not outlive main thread");
    }
}

fn wrapped_run<T: ControllerTransport>(
    session: SessionHandshake<T>,
    tx: &Sender<SessionMessage>,
    rx: Receiver<AppAction>,
) -> anyhow::Result<()> {
    let session = session.complete()?;

    // --- receive schemas ---
    let mut schemas = HashMap::new();
    let session = session.sync(|schema| {
        if schemas.insert(schema.identification, schema).is_some() {
            return Err(SchemaSyncError::DuplicateItem);
        }

        Ok(())
    })?;

    tx.send(SessionMessage::Schemas(schemas)).expect("msg");

    // --- receive init events ---
    let mut session = session.complete(|event| {
        tx.send(SessionMessage::InitEvent(event))
            .expect("should exist");
        Ok(())
    })?;

    // --- exchange events and keep ui thread free ---
    loop {
        let report = session.recv_report()?;
        tx.send(SessionMessage::Report(Box::new(report)))
            .expect("should not outlive main thread");

        match rx.try_recv() {
            Ok(action) => match action {
                    AppAction::NoAction => {},
                    AppAction::SetConfig { .. } => {},
                    AppAction::ExecuteCommand { machine, resource } => {
                        let _ = session.send_request(RuntimeRequest { 
                        request_id: 0,
                        kind: RuntimeRequestKind::InvokeMachineCommand { 
                            target: machine,
                            resource,
                        }
                    });
                }
            },
            Err(_) => continue,
        }
    }
}
