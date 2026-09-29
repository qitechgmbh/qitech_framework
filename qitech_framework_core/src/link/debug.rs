use std::thread;

use crate::link::protocol::ControllerMessage;
use crate::link::protocol::Hello;
use crate::link::protocol::RuntimeMessage;
use crate::link::transport::RuntimeListener;
use crate::link::transport::RuntimeTransport;
use crate::link::transport::TransportError;

#[derive(Debug, Default)]
pub struct DebugRuntimeListener;

impl DebugRuntimeListener {
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeListener for DebugRuntimeListener {
    type Transport = DebugRuntimeTransport;

    fn accept(&mut self) -> Result<Self::Transport, TransportError> {
        Ok(DebugRuntimeTransport {
            state: State::SendHello,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    SendHello,
    AwaitHelloAck,
    SendStart,
    Running,
}

#[derive(Debug)]
pub struct DebugRuntimeTransport {
    state: State,
}

impl RuntimeTransport for DebugRuntimeTransport {
    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        match self.state {
            State::SendHello => {
                self.state = State::AwaitHelloAck;
                Ok(ControllerMessage::Hello(Hello::new()))
            }

            State::SendStart => {
                self.state = State::Running;
                Ok(ControllerMessage::Start)
            }

            // a real controller would stay silent until the ack arrives
            // and never sends requests here, so block like a quiet connection
            State::AwaitHelloAck | State::Running => loop {
                thread::park();
            },
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        match msg {
            RuntimeMessage::HelloAck(info) => {
                let machines: Vec<_> = info.schemas.iter().map(|s| s.identification).collect();
                println!("[debug link] hello ack, machines: {machines:?}");

                if self.state == State::AwaitHelloAck {
                    self.state = State::SendStart;
                }
            }

            RuntimeMessage::HelloError(e) => println!("[debug link] hello error: {e}"),

            RuntimeMessage::UnexpectedMessage { expected } => {
                println!("[debug link] unexpected message, expected: {expected}")
            }

            RuntimeMessage::Report(report) => println!(
                "[debug link] report at {}: {} responses, {} events, {} logs, {} measurements",
                report.timestamp,
                report.responses.len(),
                report.events.len(),
                report.logs.len(),
                report.machines.measurement_snapshots.len(),
            ),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::protocol::RuntimeInfo;

    #[test]
    fn plays_hello_then_start() {
        let mut transport = DebugRuntimeListener::new().accept().unwrap();

        let ControllerMessage::Hello(hello) = transport.recv().unwrap() else {
            panic!("expected hello");
        };
        hello.validate().unwrap();

        transport
            .send(RuntimeMessage::HelloAck(RuntimeInfo {
                schemas: Vec::new(),
            }))
            .unwrap();

        assert!(matches!(
            transport.recv().unwrap(),
            ControllerMessage::Start
        ));
        assert_eq!(transport.state, State::Running);
    }
}
