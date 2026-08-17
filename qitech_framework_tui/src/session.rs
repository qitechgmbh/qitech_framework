use std::collections::HashMap;

use crossbeam::channel::Receiver;
use crossbeam::channel::Sender;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::ControllerTransport;
use qitech_framework_core::session::controller::SessionHandshake;
use qitech_framework_core::session::error::SchemaSyncError;

pub enum SessionMessage {
    Schemas(HashMap<MachineIdentification, MachineSchema>),
    InitEvent(RuntimeInitEvent),
    Report(Box<RuntimeReport>),
    Disconnected,
}

pub async fn run<T: ControllerTransport>(
    session: SessionHandshake<T>,
    tx: Sender<SessionMessage>,
    rx: Receiver<RuntimeRequest>,
) {
    if wrapped_run(session, &tx, rx).await.is_err() {
        tx.send(SessionMessage::Disconnected)
            .expect("should not outlive main thread");
    }
}

async fn wrapped_run<T: ControllerTransport>(
    session: SessionHandshake<T>,
    tx: &Sender<SessionMessage>,
    rx: Receiver<RuntimeRequest>,
) -> anyhow::Result<()> {
    let session = session.complete().await?;

    // --- receive schemas ---
    let mut schemas = HashMap::new();
    let session = session
        .sync(|schema| {
            if schemas.insert(schema.identification, schema).is_some() {
                return Err(SchemaSyncError::DuplicateItem);
            }

            Ok(())
        })
        .await?;

    tx.send(SessionMessage::Schemas(schemas)).expect("msg");

    // --- receive init events ---
    let mut session = session
        .complete(|event| {
            tx.send(SessionMessage::InitEvent(event))
                .expect("should exist");
        })
        .await?;

    // --- exchange events and keep ui thread free ---
    loop {
        let report = session.recv_report().await?;
        tx.send(SessionMessage::Report(Box::new(report)))
            .expect("should not outlive main thread");

        match rx.try_recv() {
            Ok(request) => session.send_request(request).await.expect("idk"),
            Err(_) => continue,
        }
    }
}
