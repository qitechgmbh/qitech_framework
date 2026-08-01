// --- runtime simple ---
pub struct DebugRuntimeTransport {
    state: u8,
}

impl DebugRuntimeTransport {
    pub fn start_session() -> session::SendHello<Self> {
        session::SendHello::new(Self { state: 0 })
    }
}

impl Transport for DebugRuntimeTransport {
    type In = HandleMessage;
    type Out = RuntimeMessage;

    fn recv(&mut self) -> Result<Self::In, TransportError> {
        match self.state {
            0 => {
                self.state = 1;
                Ok(HandleMessage::HelloAck)
            }
            1 => {
                self.state = 1;
                Ok(HandleMessage::SchemaAck)
            }
            _ => unreachable!(),
        }
    }

    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError> {
        Ok(None)
    }

    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError> {
        match msg {
            RuntimeMessage::Hello(hello) => println!("{hello:#?}"),
            RuntimeMessage::Schema(schema) => {
                println!("sending schema for: {:#?}", schema.identification)
            }
            RuntimeMessage::InitEvent(event) => println!("{event:#?}"),
            RuntimeMessage::Finished => println!("finished"),
            RuntimeMessage::Report(report) => {
                println!("sending report: {:#?}", report.machines.measurements);
            }
        }

        // println!("Sending: {msg:#?}");
        _ = msg;
        Ok(())
    }
}

impl RuntimeTransport for DebugRuntimeTransport {}
