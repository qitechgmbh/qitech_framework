use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixListener as SyncUnixListener;
use std::os::unix::net::UnixStream as SyncUnixStream;
use std::path::Path;

use bytes::Buf;
use bytes::BytesMut;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream as AsyncUnixStream;

use crate::session::controller_async;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::runtime;
use crate::session::transport::AsyncControllerTransport;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

// --- runtime ----
pub fn runtime(
    path: impl AsRef<Path>,
) -> Result<runtime::SessionHandshake<UnixRuntimeTransport>, TransportError> {
    let path = path.as_ref();

    // Remove stale socket.
    let _ = fs::remove_file(path);

    let listener = SyncUnixListener::bind(path)?;

    let (stream, _) = listener.accept()?;

    Ok(runtime::SessionHandshake::new(UnixRuntimeTransport::new(
        stream,
    )))
}

pub struct UnixRuntimeTransport {
    stream: SyncUnixStream,
    codec: Codec,
    blocking: bool,
}

impl UnixRuntimeTransport {
    pub fn new(stream: SyncUnixStream) -> Self {
        Self {
            stream,
            codec: Codec::new(),
            blocking: true,
        }
    }

    fn read_into_codec(&mut self) -> Result<(), TransportError> {
        let mut buffer = [0u8; 4096];

        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => return Err(TransportError::Disconnected),

                Ok(n) => {
                    self.codec.feed(&buffer[..n]);

                    // In blocking mode one read is enough.
                    // In non-blocking mode keep draining until WouldBlock.
                    if self.blocking {
                        break;
                    }
                }

                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    if self.blocking {
                        continue;
                    } else {
                        break;
                    }
                }

                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }
}

impl RuntimeTransport for UnixRuntimeTransport {
    fn set_blocking(&mut self, blocking: bool) -> Result<(), TransportError> {
        self.blocking = blocking;
        self.stream.set_nonblocking(!blocking)?;

        Ok(())
    }

    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        loop {
            if let Some(message) = self.codec.decode::<ControllerMessage>()? {
                return Ok(message);
            }

            self.read_into_codec()?;
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        let frame = Codec::encode(&msg)?;
        self.stream.write_all(&frame)?;
        self.stream.flush()?;
        Ok(())
    }
}

// --- controller async ---
pub async fn controller_tokio(
    path: impl AsRef<Path>,
) -> Result<controller_async::SessionHandshake<UnixTokioControllerTransport>, TransportError> {
    let stream = AsyncUnixStream::connect(path).await?;

    Ok(controller_async::SessionHandshake::new(
        UnixTokioControllerTransport::new(stream),
    ))
}

pub struct UnixTokioControllerTransport {
    stream: AsyncUnixStream,
    codec: Codec,
}

impl UnixTokioControllerTransport {
    pub fn new(stream: AsyncUnixStream) -> Self {
        Self {
            stream,
            codec: Codec::new(),
        }
    }
}

impl AsyncControllerTransport for UnixTokioControllerTransport {
    async fn recv(&mut self) -> Result<RuntimeMessage, TransportError> {
        loop {
            if let Some(message) = self.codec.decode::<RuntimeMessage>()? {
                return Ok(message);
            }

            let mut buffer = [0u8; 4096];

            let n = self.stream.read(&mut buffer).await?;

            if n == 0 {
                return Err(TransportError::Disconnected);
            }

            self.codec.feed(&buffer[..n]);
        }
    }

    async fn send(&mut self, msg: ControllerMessage) -> Result<(), TransportError> {
        let frame = Codec::encode(&msg)?;

        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;

        Ok(())
    }
}

// --- codec ---
const HEADER_SIZE: usize = 4;

pub struct Codec {
    rx: BytesMut,
}

impl Codec {
    pub fn new() -> Self {
        Self {
            rx: BytesMut::new(),
        }
    }

    pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, TransportError> {
        let payload = postcard::to_allocvec(value)
            .map_err(|e| TransportError::MalformedMessage(e.to_string()))?;

        let len = payload.len();

        if len > u32::MAX as usize {
            return Err(TransportError::MalformedMessage("message too large".into()));
        }

        let mut frame = Vec::with_capacity(HEADER_SIZE + len);

        frame.extend_from_slice(&(len as u32).to_be_bytes());
        frame.extend_from_slice(&payload);

        Ok(frame)
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.rx.extend_from_slice(bytes);
    }

    pub fn decode<T: DeserializeOwned>(&mut self) -> Result<Option<T>, TransportError> {
        // Need at least the length prefix
        if self.rx.len() < HEADER_SIZE {
            return Ok(None);
        }

        let len = u32::from_be_bytes([self.rx[0], self.rx[1], self.rx[2], self.rx[3]]) as usize;

        // Wait for the complete frame
        if self.rx.len() < HEADER_SIZE + len {
            return Ok(None);
        }

        // Remove header
        self.rx.advance(HEADER_SIZE);

        // Extract payload
        let payload = self.rx.split_to(len);

        let value = postcard::from_bytes(&payload)
            .map_err(|e| TransportError::MalformedMessage(e.to_string()))?;

        Ok(Some(value))
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}
