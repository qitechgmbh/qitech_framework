use std::fs;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio::time::MissedTickBehavior;

use crate::modbus::ModbusError;
use crate::modbus::ModbusEvent;
use crate::modbus::ModbusRequest;
use crate::modbus::ModbusRtuLineConfig;
use crate::modbus::ModbusRtuLineId;
use crate::modbus::RequestId;
use crate::modbus::line;
use crate::modbus::line::LineRequest;

/// Runs the manager until the runtime drops its end of `requests`.
pub(crate) async fn run(
    lines: Vec<ModbusRtuLineConfig>,
    scan_interval: Duration,
    mut requests: mpsc::Receiver<ModbusRequest>,
    events: mpsc::Sender<ModbusEvent>,
) {
    let mut lines: Vec<Line> = lines
        .into_iter()
        .map(|config| Line {
            config,
            worker: None,
        })
        .collect();

    // workers report a failed port here, so the line can be closed and reopened. Each worker
    // reports at most once, and the manager never waits on a worker, so a full channel only
    // delays the reporting worker
    let (failed_tx, mut failed_rx) = mpsc::channel(lines.len().max(1));

    let mut scan_timer = tokio::time::interval(scan_interval);
    scan_timer.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = scan_timer.tick() => scan(&mut lines, &events, &failed_tx).await,
            Some(line) = failed_rx.recv() => disconnect(&mut lines, line, &events).await,
            request = requests.recv() => match request {
                Some(request) => route(&lines, request, &events).await,
                // the runtime dropped its handle; dropping the lines closes every worker
                None => return,
            },
        }
    }
}

struct Line {
    config: ModbusRtuLineConfig,
    /// Set while the line is connected.
    worker: Option<Worker>,
}

struct Worker {
    requests: mpsc::Sender<LineRequest>,
}

/// Opens lines whose device appeared and closes lines whose device vanished.
///
/// A device that is replugged between two scans is not seen here. Its old port fails on the
/// next request, the worker reports it, and the following scan opens the line again.
async fn scan(
    lines: &mut [Line],
    events: &mpsc::Sender<ModbusEvent>,
    failed: &mpsc::Sender<ModbusRtuLineId>,
) {
    for i in 0..lines.len() {
        let id = ModbusRtuLineId(i);
        let resolved = fs::canonicalize(&lines[i].config.path).ok();

        match resolved {
            None => disconnect(lines, id, events).await,
            Some(path) if lines[i].worker.is_none() => {
                let line = &mut lines[i];
                match line::spawn(id, &line.config, &path, events.clone(), failed.clone()) {
                    Ok(requests) => {
                        line.worker = Some(Worker { requests });
                        _ = events
                            .send(ModbusEvent::LineConnected { line: id, path })
                            .await;
                    }
                    Err(err) => tracing::warn!("cannot open modbus line {path:?}: {err}"),
                }
            }
            Some(_) => {}
        }
    }
}

/// Closes a line. Its worker fails whatever is still queued and then exits.
async fn disconnect(lines: &mut [Line], id: ModbusRtuLineId, events: &mpsc::Sender<ModbusEvent>) {
    if lines[id.0].worker.take().is_some() {
        _ = events
            .send(ModbusEvent::LineDisconnected { line: id })
            .await;
    }
}

/// Hands a request to its line, or answers it right away if the line can't take it.
///
/// Never waits on a line's queue: a worker can be waiting on the manager to read `failed`, so
/// waiting here could deadlock.
async fn route(lines: &[Line], request: ModbusRequest, events: &mpsc::Sender<ModbusEvent>) {
    let line_request = LineRequest {
        id: request.id,
        slave_id: request.slave_id,
        request: request.request,
    };

    let Some(worker) = &lines[request.line.0].worker else {
        return reject(line_request.id, ModbusError::LineUnavailable, events).await;
    };

    match worker.requests.try_send(line_request) {
        Ok(()) => {}
        Err(TrySendError::Full(r)) => reject(r.id, ModbusError::QueueFull, events).await,
        Err(TrySendError::Closed(r)) => reject(r.id, ModbusError::LineUnavailable, events).await,
    }
}

async fn reject(id: RequestId, error: ModbusError, events: &mpsc::Sender<ModbusEvent>) {
    let response = ModbusEvent::Response {
        id,
        result: Err(error),
    };
    _ = events.send(response).await;
}
