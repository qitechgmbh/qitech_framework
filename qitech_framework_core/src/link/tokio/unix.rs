use std::fs;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::link::codec;
use crate::link::codec::FrameDecoder;
use crate::link::protocol::ControllerMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::transport::ControllerConnector;
use crate::link::transport::ControllerTransport;
use crate::link::transport::RuntimeListener;
use crate::link::transport::RuntimeTransport;
use crate::link::transport::TransportError;

const READ_CHUNK_SIZE: usize = 64 * 1024;

// --- runtime ---
pub struct UnixRuntimeListener {
    listener: UnixListener,
    path: PathBuf,
}

impl UnixRuntimeListener {
    /// Binds a socket at `path`. A stale socket left by a previous run is replaced,
    /// any other file at `path` is left alone and binding fails.
    pub fn bind(path: impl AsRef<Path>) -> Result<Self, TransportError> {
        let path = path.as_ref().to_path_buf();

        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_socket() => fs::remove_file(&path)?,
            Ok(_) => return Err(io::Error::from(ErrorKind::AlreadyExists).into()),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }

        let listener = UnixListener::bind(&path)?;

        Ok(Self { listener, path })
    }
}

impl RuntimeListener for UnixRuntimeListener {
    type Transport = UnixRuntimeTransport;

    fn accept(&mut self) -> Result<Self::Transport, TransportError> {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => return Ok(UnixRuntimeTransport::new(stream)),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }
}

impl Drop for UnixRuntimeListener {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub struct UnixRuntimeTransport {
    stream: UnixStream,
    decoder: FrameDecoder,
    chunk: Box<[u8]>,
}

impl UnixRuntimeTransport {
    fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            // the controller has to open with a hello
            decoder: FrameDecoder::new(codec::HELLO_FRAME_LIMIT),
            chunk: vec![0u8; READ_CHUNK_SIZE].into_boxed_slice(),
        }
    }
}

impl RuntimeTransport for UnixRuntimeTransport {
    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        loop {
            if let Some(msg) = self.decoder.decode()? {
                return Ok(msg);
            }

            match self.stream.read(&mut self.chunk) {
                Ok(0) => return Err(TransportError::Disconnected),
                Ok(n) => self.decoder.feed(&self.chunk[..n]),
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(map_io_err(e)),
            }
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        let frame = codec::encode(&msg)?;
        self.stream.write_all(&frame).map_err(map_io_err)
    }
}

// --- controller ---
#[derive(Debug, Clone)]
pub struct UnixControllerConnector {
    path: PathBuf,
}

impl UnixControllerConnector {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl ControllerConnector for UnixControllerConnector {
    type Transport = UnixControllerTransport;

    async fn connect(&mut self) -> Result<Self::Transport, TransportError> {
        let stream = tokio::net::UnixStream::connect(&self.path).await?;

        Ok(UnixControllerTransport {
            stream,
            // the runtime answers with its schemas, so no small first frame limit here
            decoder: FrameDecoder::new(codec::FRAME_LIMIT),
            chunk: vec![0u8; READ_CHUNK_SIZE].into_boxed_slice(),
        })
    }
}

pub struct UnixControllerTransport {
    stream: tokio::net::UnixStream,
    decoder: FrameDecoder,
    chunk: Box<[u8]>,
}

impl ControllerTransport for UnixControllerTransport {
    async fn recv(&mut self) -> Result<RuntimeMessage, TransportError> {
        loop {
            if let Some(msg) = self.decoder.decode()? {
                return Ok(msg);
            }

            // cancel safe: bytes only reach the decoder after the read completed
            let n = self.stream.read(&mut self.chunk).await.map_err(map_io_err)?;

            if n == 0 {
                return Err(TransportError::Disconnected);
            }

            self.decoder.feed(&self.chunk[..n]);
        }
    }

    async fn send(&mut self, msg: ControllerMessage) -> Result<(), TransportError> {
        let frame = codec::encode(&msg)?;
        self.stream.write_all(&frame).await.map_err(map_io_err)
    }
}

/// Maps errors that mean the peer went away to `Disconnected`.
fn map_io_err(e: io::Error) -> TransportError {
    match e.kind() {
        ErrorKind::BrokenPipe | ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
            TransportError::Disconnected
        }
        _ => TransportError::Io(e),
    }
}
