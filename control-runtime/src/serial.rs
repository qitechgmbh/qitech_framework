use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio_serial::{SerialPortInfo, available_ports};

pub async fn run_scanner(
    serial_scan_interval: Duration,
    tx: mpsc::Sender<Vec<SerialPortInfo>>
) {
    let mut last_scan = Instant::now();

    loop {
        let now = Instant::now();
        if now.duration_since(last_scan) < serial_scan_interval {
            continue;
        }

        last_scan = now;

        let Ok(ports) = available_ports() else { continue; };
        
        if (tx.send(ports).await).is_err() {
            // channel closed, finish
            return ;
        };
    }
}
