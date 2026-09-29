#![cfg(feature = "link_tokio")]

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;

use qitech_framework_core::link::ControllerConnector;
use qitech_framework_core::link::ControllerTransport;
use qitech_framework_core::link::RuntimeListener;
use qitech_framework_core::link::RuntimeTransport;
use qitech_framework_core::link::TransportError;
use qitech_framework_core::link::protocol::ControllerMessage;
use qitech_framework_core::link::protocol::Hello;
use qitech_framework_core::link::protocol::RuntimeInfo;
use qitech_framework_core::link::protocol::RuntimeMessage;
use qitech_framework_core::link::tokio::mpsc;
use qitech_framework_core::link::tokio::unix::UnixControllerConnector;
use qitech_framework_core::link::tokio::unix::UnixRuntimeListener;

fn socket_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("qitech-link-{}-{name}.sock", std::process::id()))
}

/// Runtime side of one connection: answer the hello, then wait for start or abort.
/// Returns the message that ended the handshake.
fn runtime_handshake(transport: &mut impl RuntimeTransport) -> ControllerMessage {
    let ControllerMessage::Hello(hello) = transport.recv().unwrap() else {
        panic!("expected hello");
    };
    hello.validate().unwrap();

    transport
        .send(RuntimeMessage::HelloAck(RuntimeInfo {
            schemas: Vec::new(),
        }))
        .unwrap();

    transport.recv().unwrap()
}

async fn controller_handshake(transport: &mut impl ControllerTransport, then: ControllerMessage) {
    transport
        .send(ControllerMessage::Hello(Hello::new()))
        .await
        .unwrap();

    assert!(matches!(
        transport.recv().await.unwrap(),
        RuntimeMessage::HelloAck(_)
    ));

    transport.send(then).await.unwrap();
}

/// The runtime accepts a connection that aborts, goes back to listening and accepts the next one.
fn runtime_abort_then_start(mut listener: impl RuntimeListener) {
    let mut first = listener.accept().unwrap();
    assert!(matches!(
        runtime_handshake(&mut first),
        ControllerMessage::Abort { .. }
    ));
    assert!(matches!(first.recv(), Err(TransportError::Disconnected)));

    let mut second = listener.accept().unwrap();
    assert!(matches!(
        runtime_handshake(&mut second),
        ControllerMessage::Start
    ));

    second
        .send(RuntimeMessage::UnexpectedMessage {
            expected: "test".into(),
        })
        .unwrap();
}

async fn controller_abort_then_start(mut connector: impl ControllerConnector) {
    let mut first = connector.connect().await.unwrap();
    controller_handshake(
        &mut first,
        ControllerMessage::Abort {
            reason: "schema rejected".into(),
        },
    )
    .await;
    drop(first);

    let mut second = connector.connect().await.unwrap();
    controller_handshake(&mut second, ControllerMessage::Start).await;

    assert!(matches!(
        second.recv().await.unwrap(),
        RuntimeMessage::UnexpectedMessage { expected } if expected == "test"
    ));
}

#[tokio::test]
async fn mpsc_abort_then_reconnect() {
    let (listener, connector) = mpsc::link(8);

    let runtime = thread::spawn(move || runtime_abort_then_start(listener));
    controller_abort_then_start(connector).await;

    runtime.join().unwrap();
}

#[tokio::test]
async fn unix_abort_then_reconnect() {
    let path = socket_path("reconnect");
    let listener = UnixRuntimeListener::bind(&path).unwrap();

    let runtime = thread::spawn(move || runtime_abort_then_start(listener));
    controller_abort_then_start(UnixControllerConnector::new(&path)).await;

    runtime.join().unwrap();
    assert!(!path.exists(), "listener removes its socket on drop");
}

#[test]
fn unix_rejects_oversized_first_frame() {
    let path = socket_path("oversized");
    let mut listener = UnixRuntimeListener::bind(&path).unwrap();

    let mut peer = UnixStream::connect(&path).unwrap();
    peer.write_all(&u32::MAX.to_be_bytes()).unwrap();

    let mut transport = listener.accept().unwrap();
    assert!(matches!(
        transport.recv(),
        Err(TransportError::FrameTooLarge { .. })
    ));
}

#[test]
fn unix_bind_does_not_replace_regular_file() {
    let path = socket_path("regular-file");
    std::fs::write(&path, b"keep me").unwrap();

    assert!(UnixRuntimeListener::bind(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"keep me");

    std::fs::remove_file(&path).unwrap();
}
