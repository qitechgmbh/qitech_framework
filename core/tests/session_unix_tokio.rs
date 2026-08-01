use std::thread;
use std::time::Duration;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::report::TimingsReport;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::session;

#[test]
fn main() {
    let socket_path = "/tmp/just_some_sock.sock";

    let runtime_thread = thread::spawn({
        let socket_path = socket_path.to_string();
        move || {
            runtime(socket_path);
        }
    });

    thread::sleep(Duration::from_millis(100));

    let controller_thread = thread::spawn({
        let socket_path = socket_path.to_string();
        move || {
            controller(socket_path);
        }
    });

    controller_thread.join().unwrap();
    runtime_thread.join().unwrap();
}

fn runtime(path: String) {
    let session = session::unix::runtime(&path).expect("runtime transport failed");

    // --- initial handshake ---
    let session = session.complete().expect("runtime handshake failed");

    // --- sync schemas ---
    // TODO:

    // --- send init events ---
    let mut session = session
        .complete()
        .expect("[Runtime] couldn't complete schema sync phase");

    session
        .send_event(RuntimeInitEvent::EtherCATDiscoveryStarted)
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATDiscoveryCompleted {
            interface: "just_some_interface".to_string(),
        })
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATStateUpdate(EtherCATStatus::Boot))
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATStateUpdate(EtherCATStatus::Init))
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATStateUpdate(EtherCATStatus::PreOp))
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATStateUpdate(
            EtherCATStatus::PreopPdi,
        ))
        .unwrap();

    session
        .send_event(RuntimeInitEvent::EtherCATFinalizing)
        .unwrap();

    // --- go into running stage ---
    let mut session = session
        .complete()
        .expect("[Runtime] couldn't complete schema sync phase");

    loop {
        match session.recv_request() {
            Ok(Some(request)) => {
                println!("[Runtime] received request: {request:?}");

                let response = (request.transaction_id, Ok(()));

                let report = RuntimeReport {
                    timestamp: Utc::now(),
                    responses: vec![response],
                    timings: TimingsReport::default(),
                    machines: Default::default(),
                    events: Default::default(),
                    logs: Default::default(),
                };

                session
                    .send_report(report)
                    .expect("[Runtime] Couldn't send report");

                break;
            }

            Ok(None) => {
                println!("[Runtime] idle");
            }

            Err(error) => {
                eprintln!("[Runtime] error: {error}");
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn controller(path: String) {
    let runtime = tokio::runtime::Runtime::new().expect("[Controller] tokio runtime failed");

    runtime.block_on(async {
        let session = session::unix::controller_tokio(&path)
            .await
            .expect("[Controller] transport failed");

        // --- process hello ---
        let session = session
            .complete()
            .await
            .expect("[Controller] handshake failed");

        // --- sync schemas ---
        let session = session
            .sync(|schema| {
                println!("[Controller] Received schema for {}", schema.identification);
                Ok(())
            })
            .await
            .expect("[Controller] handshake failed");

        // --- receive init events ---
        let mut session = session
            .complete(|event| {
                println!("[Controller] Received event: {event:?}");
                Ok(())
            })
            .await
            .expect("[Controller] handshake failed");

        // --- send request ---
        session
            .send_request(RuntimeRequest {
                transaction_id: 0,
                kind: RuntimeRequestKind::SetMachineConfiguration {
                    target: MachineIdentificationUnique {
                        identification: MachineIdentification {
                            vendor_id: 0,
                            machine_id: 0,
                        },
                        serial: 0,
                    },
                    resource: "just_some_config".to_string(),
                    value: ScalarValue::Float(Some(1.0)),
                },
            })
            .await
            .expect("[Controller] send failed");

        let report = session
            .recv_report()
            .await
            .expect("[Controller] couldn't recv report");

        println!("received report: {report:#?}");
    });
}
