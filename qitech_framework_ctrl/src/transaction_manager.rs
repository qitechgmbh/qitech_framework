use std::collections::HashMap;

use qitech_framework_core::request::RuntimeRequest;
use tokio::sync::mpsc;

use crate::RuntimeReportReceiver;
use crate::RuntimeRequestReceiver;
use crate::RuntimeRequestResponder;

#[tracing::instrument(skip_all)]
pub async fn run(
    mut report_rx: RuntimeReportReceiver,
    mut request_receiver: RuntimeRequestReceiver,
    mut request_dispatcher_rx: mpsc::Receiver<mpsc::Sender<RuntimeRequest>>,
) {
    let mut request_counter = 0u64;
    let mut request_tx = None;
    let mut pending: HashMap<u64, RuntimeRequestResponder> = HashMap::new();

    loop {
        tokio::select! {
            biased;

            sender = request_dispatcher_rx.recv() => {
                let sender = sender.expect("");
                request_tx = Some(sender);
            }

            request = request_receiver.recv() => {
                let Some((kind, responder)) = request else {
                    // no more request senders, exit
                    return;
                };

                let Some(tx) = request_tx.as_mut() else {
                    // drop responder to signal that no request can be made
                    _ = responder;
                    continue;
                };

                let request_id = request_counter;
                request_counter += 1;

                let request = RuntimeRequest { request_id, kind };

                if tx.send(request).await.is_err() {
                    // session was terminated, cannot send new requests.
                    // But can still respond to pending ones
                    request_tx = None;
                    continue;
                }

                pending.insert(request_id, responder);
            }

            report = report_rx.recv() => {
                let report = report.expect("dropped report_tx");

                for response in &report.responses {
                    let Some(tx) = pending.remove(&response.request_id) else {
                        continue;
                    };

                    _ = tx.send(response.result.clone());
                }
            }
        }
    }
}
