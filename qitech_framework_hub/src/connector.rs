use qitech_framework_core::session::ControllerTransport;
use qitech_framework_core::session::error::TransportError;

pub trait TransportConnector: Send + Sync {
    type Transport: ControllerTransport;
    fn connect(&mut self) -> impl Future<Output = Result<Self::Transport, TransportError>> + Send;
}

// impl TransportConnector for SessionHandshake<TokioMpscControllerTransport> {
//     type Transport = TokioMpscControllerTransport;
//
//     fn connect(&mut self) -> impl Future<Output = Result<Self::Transport, TransportError>> + Send {
//         todo!()
//     }
// }
