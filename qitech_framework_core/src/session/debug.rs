use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::runtime;
use crate::session::runtime::SessionHandshake;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

pub fn runtime() -> runtime::SessionHandshake<DebugRuntimeTransport> {
    let transport = DebugRuntimeTransport { state: 0 };
    SessionHandshake::new(transport)
}

pub struct DebugRuntimeTransport {
    state: u8,
}

impl RuntimeTransport for DebugRuntimeTransport {
    fn set_blocking(&mut self, blocking: bool) -> Result<(), TransportError> {
        _ = blocking;
        Ok(())
    }

    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        match self.state {
            0 => Ok(ControllerMessage::HelloAck),
            1 => Ok(ControllerMessage::SchemaAck),
            _ => Err(TransportError::WouldBlock),
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        match msg {
            RuntimeMessage::Hello(hello) => {
                self.state = 0;
                println!("{hello:#?}")
            }
            RuntimeMessage::Schema(schema) => {
                self.state = 1;
                println!("sending schema for: {:#?}", schema.identification)
            }
            RuntimeMessage::InitEvent(event) => {
                self.state = 2;
                println!("{event:#?}")
            }
            RuntimeMessage::Finished => println!("finished"),
            RuntimeMessage::Report(report) => {
                println!(
                    "sending report: {:#?}",
                    report.machines.measurement_snapshots
                );
            }
        }

        Ok(())
    }
}
