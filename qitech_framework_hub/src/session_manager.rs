use std::sync::Arc;

use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::session::ControllerSessionProvider;
use qitech_framework_core::session::ControllerTransport;
use qitech_framework_core::session::error::SchemaSyncError;
use tokio::sync::mpsc;

use crate::types::RuntimeReportSender;
use crate::types::SchemaRegistry;
use crate::types::Swappable;

#[tracing::instrument(skip_all)]
pub async fn run<T: ControllerTransport>(
    mut provider: impl ControllerSessionProvider<Transport = T>,
    schema_registry: Swappable<SchemaRegistry>,
    report_sender: RuntimeReportSender,
    request_dispatcher_tx: mpsc::Sender<mpsc::Sender<RuntimeRequest>>,
) {
    loop {
        tracing::info!("connecting to runtime");

        let session = match provider.provide().await {
            Ok(connection) => {
                tracing::info!("runtime connection established");
                connection
            }
            Err(err) => {
                tracing::error!(%err, "failed to connect to runtime");
                continue;
            }
        };

        let session = match session.complete().await {
            Ok(session) => {
                tracing::info!("runtime handshake completed");
                session
            }
            Err(err) => {
                tracing::warn!(%err, "runtime handshake failed");
                continue;
            }
        };

        tracing::debug!("synchronizing runtime schemas");

        let mut schemas = (*schema_registry.load_full()).clone();

        let session = match session
            .sync(|schema| {
                tracing::debug!(
                    ?schema.identification,
                    "received runtime schema"
                );

                let ident = schema.identification;

                if schemas.insert(ident, schema).is_some() {
                    tracing::warn!(?ident, "runtime sent duplicate schema");
                    return Err(SchemaSyncError::DuplicateItem);
                }

                Ok(())
            })
            .await
        {
            Ok(session) => session,
            Err(err) => {
                tracing::warn!(%err, "runtime schema synchronization failed");
                continue;
            }
        };

        let schema_count = schemas.len();
        schema_registry.store(Arc::new(schemas));

        tracing::info!(schema_count, "runtime schema synchronization completed");

        let mut session = match session
            .complete(|event| {
                tracing::debug!(?event, "received runtime initialization event");
            })
            .await
        {
            Ok(session) => session,
            Err(err) => {
                tracing::warn!(%err, "runtime initialization failed");
                continue;
            }
        };

        tracing::info!("runtime session ready");

        let (tx, mut rx) = mpsc::channel(64);
        request_dispatcher_tx
            .send(tx)
            .await
            .expect("transaction manager dropped receiver");

        loop {
            tokio::select! {
                biased;

                report = session.recv_report() => {
                    let report = match report {
                        Ok(report) => report,
                        Err(err) => {
                            tracing::warn!(%err, "runtime connection lost while receiving report");
                            break;
                        }
                    };

                    tracing::debug!(
                        response_count = report.responses.len(),
                        "received runtime report"
                    );

                    report_sender
                        .send(Arc::new(report))
                        .expect("report receiver must live for the lifetime of the program");
                }

                request = rx.recv() => {
                    let request = request.expect("transaction manager dropped sender");

                    let request_id = request.request_id;

                    if let Err(err) = session
                        .send_request(request)
                        .await
                    {
                        tracing::warn!(
                            %err,
                            request_id,
                            "failed to send request to runtime"
                        );

                        break;
                    }
                }
            }
        }

        tracing::info!("runtime session ended; reconnecting");
    }
}
