use std::pin::Pin;

use control_core::{RuntimeReport, RuntimeRequest};
use tokio::net::{UnixListener, UnixStream};

use super::{Listener, PendingConnection, SessionTransport};

pub struct UnixSocketListener {
    inner: UnixListener,
}

impl Listener for UnixSocketListener {
    type Pending = PendingUnixConnection;
    type Transport = UnixSessionTransport;

    async fn accept(&mut self) -> anyhow::Result<Self::Pending> {
        let (stream, _) = self.inner.accept().await?;

        Ok(PendingUnixConnection {
            stream,
        })
    }

    async fn upgrade(
        &self,
        conn: Self::Pending,
    ) -> anyhow::Result<Self::Transport> {
        Ok(UnixSessionTransport {
            stream: conn.stream,
        })
    }
}

pub struct PendingUnixConnection {
    stream: UnixStream,
}

impl PendingConnection for PendingUnixConnection {
    async fn recv(&mut self) -> anyhow::Result<Vec<String>> {
        let schemas = recv_message(&mut self.stream).await?;

        Ok(schemas)
    }
}

pub struct UnixSessionTransport {
    stream: UnixStream,
}

impl SessionTransport for UnixSessionTransport {
    fn recv(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<RuntimeReport>> + Send + '_>> {
        Box::pin(async move {
            recv_message(&mut self.stream).await
        })
    }

    fn send(
        &mut self,
        request: RuntimeRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            send_message(&mut self.stream, request).await
        })
    }
}

async fn recv_message<T, M>(stream: &mut T) -> anyhow::Result<M>
where
    T: tokio::io::AsyncRead + Unpin,
    M: serde::de::DeserializeOwned,
{
    // read length prefix
    // read bytes
    // deserialize
    todo!()
}

async fn send_message<T, M>(
    stream: &mut T,
    msg: M,
) -> anyhow::Result<()>
where
    T: tokio::io::AsyncWrite + Unpin,
    M: serde::Serialize,
{
    // serialize
    // write length prefix
    // write bytes
    todo!()
}
