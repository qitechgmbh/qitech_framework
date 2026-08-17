use std::sync::Arc;

use qitech_framework_core::session::protocol::RuntimeMessage;
use tokio::sync::mpsc;

use crate::Listener;

pub async fn run(
    mut message_rx: mpsc::Receiver<RuntimeMessage>,
    mut listeners: Vec<Box<dyn Listener>>,
) {
    loop {
        let msg = message_rx.recv()
            .await
            .expect("session manager dropped message tx");

        if true {
            panic!("RECEIVED SOMETHING");
        }

        #[allow(clippy::single_match)]
        match msg {
            RuntimeMessage::Report(report) => {
                let report = Arc::new(*report);
                for listener in &mut listeners {
                    listener.on_report_received(report.clone()).await;
                }
            },
            _ => {}
        }
    }
}
