//! Framing for byte stream transports.
//!
//! Every frame is a 4 byte big endian payload length followed by a postcard encoded payload.
//!
//! NOTE: The header layout is part of the hello exchange and must stay stable.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::link::transport::TransportError;

const HEADER_SIZE: usize = 4;

/// Limit for the first frame of a connection when it is expected to be a `Hello`.
/// Keeps a foreign peer from making us buffer a huge frame before the magic is checked.
pub(crate) const HELLO_FRAME_LIMIT: usize = 1024;

/// Limit for all other frames.
pub(crate) const FRAME_LIMIT: usize = 64 * 1024 * 1024;

pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, TransportError> {
    let mut frame = postcard::to_extend(value, vec![0u8; HEADER_SIZE])
        .map_err(|e| TransportError::MalformedMessage(e.to_string()))?;

    let len = frame.len() - HEADER_SIZE;

    if len > FRAME_LIMIT {
        return Err(TransportError::FrameTooLarge {
            len,
            max: FRAME_LIMIT,
        });
    }

    frame[..HEADER_SIZE].copy_from_slice(&(len as u32).to_be_bytes());

    Ok(frame)
}

/// Collects incoming bytes and splits them into frames.
///
/// After `FrameTooLarge` the stream can't be resynchronized and the connection has to be dropped.
/// A `MalformedMessage` consumes its frame, so decoding can continue with the next one.
pub(crate) struct FrameDecoder {
    rx: Vec<u8>,
    limit: usize,
}

impl FrameDecoder {
    /// `first_limit` applies to the first frame only, all later frames use `FRAME_LIMIT`.
    pub(crate) fn new(first_limit: usize) -> Self {
        Self {
            rx: Vec::new(),
            limit: first_limit,
        }
    }

    pub(crate) fn feed(&mut self, bytes: &[u8]) {
        self.rx.extend_from_slice(bytes);
    }

    pub(crate) fn decode<T: DeserializeOwned>(&mut self) -> Result<Option<T>, TransportError> {
        if self.rx.len() < HEADER_SIZE {
            return Ok(None);
        }

        let len = u32::from_be_bytes([self.rx[0], self.rx[1], self.rx[2], self.rx[3]]) as usize;

        if len > self.limit {
            return Err(TransportError::FrameTooLarge {
                len,
                max: self.limit,
            });
        }

        let end = HEADER_SIZE + len;

        if self.rx.len() < end {
            return Ok(None);
        }

        let result = postcard::from_bytes(&self.rx[HEADER_SIZE..end]);

        self.rx.drain(..end);
        self.limit = FRAME_LIMIT;

        result
            .map(Some)
            .map_err(|e| TransportError::MalformedMessage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_frames_fed_in_pieces() {
        let mut bytes = encode(&42u32).unwrap();
        bytes.extend(encode(&"hello".to_string()).unwrap());

        let mut decoder = FrameDecoder::new(FRAME_LIMIT);

        for byte in &bytes[..bytes.len() - 1] {
            decoder.feed(&[*byte]);
        }

        assert_eq!(decoder.decode::<u32>().unwrap(), Some(42));
        assert_eq!(decoder.decode::<String>().unwrap(), None);

        decoder.feed(&bytes[bytes.len() - 1..]);

        assert_eq!(decoder.decode::<String>().unwrap(), Some("hello".into()));
        assert_eq!(decoder.decode::<String>().unwrap(), None);
    }

    #[test]
    fn rejects_oversized_first_frame_before_buffering_it() {
        let mut decoder = FrameDecoder::new(HELLO_FRAME_LIMIT);
        decoder.feed(&u32::MAX.to_be_bytes());

        assert!(matches!(
            decoder.decode::<u32>(),
            Err(TransportError::FrameTooLarge {
                max: HELLO_FRAME_LIMIT,
                ..
            })
        ));
    }

    #[test]
    fn first_frame_limit_is_lifted_afterwards() {
        let big = vec![0u8; HELLO_FRAME_LIMIT * 2];

        let mut decoder = FrameDecoder::new(HELLO_FRAME_LIMIT);
        decoder.feed(&encode(&1u8).unwrap());
        decoder.feed(&encode(&big).unwrap());

        assert_eq!(decoder.decode::<u8>().unwrap(), Some(1));
        assert_eq!(decoder.decode::<Vec<u8>>().unwrap(), Some(big));
    }

    #[test]
    fn malformed_frame_is_consumed() {
        let mut decoder = FrameDecoder::new(FRAME_LIMIT);
        decoder.feed(&encode(&"not a bool".to_string()).unwrap());
        decoder.feed(&encode(&true).unwrap());

        assert!(matches!(
            decoder.decode::<bool>(),
            Err(TransportError::MalformedMessage(_))
        ));
        assert_eq!(decoder.decode::<bool>().unwrap(), Some(true));
    }
}
