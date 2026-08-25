use qitech_framework_core::session::ControllerSessionProvider;
use qitech_framework_core::session::ControllerTransport;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

mod types;
use types::MachineRegistry;
use types::RuntimeReportReceiver;
use types::RuntimeReportSender;
use types::RuntimeRequestReceiver;
use types::RuntimeRequestResponder;
use types::RuntimeRequestSender;
use types::SchemaRegistry;
use types::Swappable;

mod config;
pub use config::HubConfiguration;

mod modules;
pub use modules::Actor;
pub use modules::ActorContext;
pub use modules::Listener;

mod session_manager;
mod transaction_manager;

pub async fn run<T: ControllerTransport + 'static>(
    config: HubConfiguration,
    provider: impl ControllerSessionProvider<Transport = T> + 'static,
) -> Result<(), i64> {
    let (request_dispatcher_tx, request_dispatcher_rx) = mpsc::channel(1024);
    // let (message_tx, message_rx) = mpsc::channel(128);
    let mut tasks = JoinSet::new();

    tasks.spawn(session_manager::run(
        provider,
        config.schemas.clone(),
        config.report_tx.clone(),
        request_dispatcher_tx,
        config.listeners,
    ));

    tasks.spawn(transaction_manager::run(
        config.report_tx.subscribe(),
        config.request_rx,
        request_dispatcher_rx,
    ));

    // tasks.spawn(listener_manager::run(
    //     message_rx,
    //     config.listeners,
    // ));

    // --- start actors ---
    for actor in config.actors {
        tasks.spawn(actor);
    }

    // --- wait for tasks to finish ---
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(()) => {}
            Err(error) => {
                // Task panicked or was cancelled.
                eprintln!("task failed: {error}");
            }
        }
    }

    Ok(())
}
