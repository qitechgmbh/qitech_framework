use std::collections::HashMap;

use crossbeam::channel::Sender;
use qitech_framework::MachineIdentification;
use qitech_framework::link::HandleTransport;
use qitech_framework::link::handle::session;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;

pub enum SessionMessage {
    Schemas(HashMap<MachineIdentification, MachineSchema>),
    InitEvent(RuntimeInitEvent),
    Finished,
    Running,
    Report(RuntimeReport),
    Disconnected,
}

pub fn run<T: HandleTransport>(session: session::ReceiveHello<T>, tx: Sender<SessionMessage>) {
    if wrapped_run(session, &tx).is_err() {
        tx.send(SessionMessage::Disconnected)
            .expect("should not outlive main thread");
    }
}

fn wrapped_run<T: HandleTransport>(
    session: session::ReceiveHello<T>,
    tx: &Sender<SessionMessage>,
) -> anyhow::Result<()> {
    let session = session.complete()?;

    // --- receive schemas ---
    let mut schemas = HashMap::new();
    let session = session.sync(|schema| {
        if schemas.insert(schema.identification, schema).is_some() {
            return Err("duplicate entry".to_string());
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
        tx.send(SessionMessage::Report(report))
            .expect("should not outlive main thread");
    }
}
