use std::time::Duration;
use tokio::sync::watch;
use control_hub::{ControlHub, Config, DatabaseConfig};

mod runtime;
use runtime::simulate_runtime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config { 
        database: DatabaseConfig {
            url: "http://localhost:8123".into(), 
            name: "control_hub".into(),
            user: "default".into(), 
            password: Some("bootstrap".into()),
        }, 
        export_interval: Duration::from_millis(2500),
        api_address: "0.0.0.0:3000".into(),
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    _ = shutdown_tx;

    // deploy ctrl + c hook for graceful shutdown
    // tokio::spawn(async move {
    //     if let Err(e) = tokio::signal::ctrl_c().await {
    //         eprintln!("failed to listen for ctrl-c: {e}");
    //         return;
    //     }
    // 
    //     println!("received ctrl-c, shutting down");
    //     // ignore error: if there are no receivers left, there's nothing to notify
    //     let _ = shutdown_tx.send(());
    // });

    // initialize a new hub instance
    let (app, session) = ControlHub::init_embedded(
        config, 
        vec![
            include_str!("machine_0.yaml").to_string(),
            include_str!("machine_1.yaml").to_string(),
        ],
        shutdown_rx,
    ).await?;

    // run app and runtime simulation
    tokio::select! {
        _ = simulate_runtime(session) => {},
        _ = app.run() => {},
    }

    Ok(())
}
