#![forbid(unsafe_code)]
//! ENKIDU TCP wire frame codec.
//!
//! Every message on the wire is an `EnkiFrame`:
//!
//! ```text
//! ┌──────────┬────────────┬──────────┬────────────────────────┐
//! │ kind (1B)│ flags (1B) │ len (4B) │ payload (len bytes)    │
//! └──────────┴────────────┴──────────┴────────────────────────┘
//! ```
//!
//! Frame kinds and their roles in the federated query pipeline:
//!
//! | Kind           | Direction        | Payload                          |
//! |----------------|------------------|----------------------------------|
//! | QUERY          | Client → Node    | HeptaScript bytes + actor KAKI   |
//! | QUERY_FRAGMENT | Node → Node      | Sub-query forwarded to peer node  |
//! | RESULT_STREAM  | Node → Client    | Batch of serialised particles     |
//! | RESULT_END     | Node → Client    | Final stats (total, duration_ms)  |
//! | ERROR          | Node → Client    | Error code + UTF-8 message        |
//! | PING / PONG    | Both directions  | Empty payload — keepalive         |

use enkidb_kaki::Kaki;

/// Maximum payload size: 4 MB per frame (avoids head-of-line blocking).
pub const MAX_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

/// Wire byte for each frame kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameKind {
    Query = 0x01,
    QueryFragment = 0x02,
    ResultStream = 0x03,
    ResultEnd = 0x04,
    Error = 0x05,
    Ping = 0x06,
    Pong = 0x07,
}

impl FrameKind {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Query),
            0x02 => Some(Self::QueryFragment),
            0x03 => Some(Self::ResultStream),
            0x04 => Some(Self::ResultEnd),
            0x05 => Some(Self::Error),
            0x06 => Some(Self::Ping),
            0x07 => Some(Self::Pong),
            _ => None,
        }
    }
}

/// A single ENKIDU wire frame (decoded from or to be encoded onto a TCP stream).
#[derive(Debug, Clone)]
pub struct EnkiFrame {
    pub kind: FrameKind,
    /// Frame-level flags (reserved, always 0x00 in v1).
    pub flags: u8,
    /// Raw payload bytes (≤ MAX_PAYLOAD_BYTES).
    pub payload: Vec<u8>,
}

impl EnkiFrame {
    /// Build a QUERY frame from raw HeptaScript bytes and the actor KAKI.
    /// Layout: [actor_kaki (16B)] [heptascript_bytes …]
    pub fn query(actor_kaki: Kaki, heptascript: &[u8]) -> Self {
        let mut payload = Vec::with_capacity(16 + heptascript.len());
        payload.extend_from_slice(actor_kaki.bytes());
        payload.extend_from_slice(heptascript);
        Self {
            kind: FrameKind::Query,
            flags: 0,
            payload,
        }
    }

    /// Build a RESULT_STREAM frame carrying a batch of raw particle bytes.
    pub fn result_stream(batch: Vec<u8>) -> Self {
        Self {
            kind: FrameKind::ResultStream,
            flags: 0,
            payload: batch,
        }
    }

    /// Build a RESULT_END frame.
    /// Payload: [total_particles (8B LE)] [duration_ms (4B LE)]
    pub fn result_end(total_particles: u64, duration_ms: u32) -> Self {
        let mut payload = Vec::with_capacity(12);
        payload.extend_from_slice(&total_particles.to_le_bytes());
        payload.extend_from_slice(&duration_ms.to_le_bytes());
        Self {
            kind: FrameKind::ResultEnd,
            flags: 0,
            payload,
        }
    }

    /// Build an ERROR frame.
    /// Payload: [error_code (1B)] [utf8_message …]
    pub fn error(code: u8, message: &str) -> Self {
        let mut payload = Vec::with_capacity(1 + message.len());
        payload.push(code);
        payload.extend_from_slice(message.as_bytes());
        Self {
            kind: FrameKind::Error,
            flags: 0,
            payload,
        }
    }

    pub fn ping() -> Self {
        Self {
            kind: FrameKind::Ping,
            flags: 0,
            payload: vec![],
        }
    }
    pub fn pong() -> Self {
        Self {
            kind: FrameKind::Pong,
            flags: 0,
            payload: vec![],
        }
    }
}

/// Stateless codec: encode `EnkiFrame` → bytes and decode bytes → `EnkiFrame`.
pub struct FrameCodec;

impl FrameCodec {
    /// Header size in bytes: kind(1) + flags(1) + len(4) = 6.
    pub const HEADER_LEN: usize = 6;

    /// Encode a frame into `out` buffer (appends; does not clear first).
    /// Returns `Err` if payload exceeds `MAX_PAYLOAD_BYTES`.
    pub fn encode(frame: &EnkiFrame, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        if frame.payload.len() > MAX_PAYLOAD_BYTES {
            return Err(EncodeError::PayloadTooLarge(frame.payload.len()));
        }
        out.push(frame.kind as u8);
        out.push(frame.flags);
        let len = frame.payload.len() as u32;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&frame.payload);
        Ok(())
    }

    /// Decode one frame from the front of `src`.
    ///
    /// Returns:
    /// - `Ok(Some((frame, consumed)))` when a complete frame is available.
    /// - `Ok(None)` when more bytes are needed.
    /// - `Err` on malformed data.
    pub fn decode(src: &[u8]) -> Result<Option<(EnkiFrame, usize)>, DecodeError> {
        if src.len() < Self::HEADER_LEN {
            return Ok(None);
        }
        let kind_byte = src[0];
        let flags = src[1];
        let payload_len = u32::from_le_bytes([src[2], src[3], src[4], src[5]]) as usize;

        if payload_len > MAX_PAYLOAD_BYTES {
            return Err(DecodeError::PayloadTooLarge(payload_len));
        }

        let total = Self::HEADER_LEN + payload_len;
        if src.len() < total {
            return Ok(None); // Incomplete — wait for more bytes.
        }

        let kind = FrameKind::from_byte(kind_byte).ok_or(DecodeError::UnknownKind(kind_byte))?;

        let payload = src[Self::HEADER_LEN..total].to_vec();
        Ok(Some((
            EnkiFrame {
                kind,
                flags,
                payload,
            },
            total,
        )))
    }
}

#[derive(Debug)]
pub enum EncodeError {
    PayloadTooLarge(usize),
}

#[derive(Debug)]
pub enum DecodeError {
    UnknownKind(u8),
    PayloadTooLarge(usize),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayloadTooLarge(n) => {
                write!(f, "payload {n}B exceeds {MAX_PAYLOAD_BYTES}B limit")
            }
        }
    }
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownKind(b) => write!(f, "unknown frame kind 0x{b:02X}"),
            Self::PayloadTooLarge(n) => {
                write!(f, "payload {n}B exceeds {MAX_PAYLOAD_BYTES}B limit")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enkidb_kaki::{derive_pattern_kaki, FixedCoord7D, PatternType};

    fn dummy_kaki() -> Kaki {
        derive_pattern_kaki(
            PatternType::CrowdFlow,
            FixedCoord7D {
                d: [1, 0, 0, 0, 0, 0, 0],
            },
            [0u8; 32],
            8_500,
            1,
        )
    }

    #[test]
    fn encode_decode_query() {
        let kaki = dummy_kaki();
        let script = b"SELECT * FROM TRIBE WHERE orbital > 5";
        let frame = EnkiFrame::query(kaki, script);

        let mut buf = Vec::new();
        FrameCodec::encode(&frame, &mut buf).unwrap();

        let (decoded, consumed) = FrameCodec::decode(&buf).unwrap().unwrap();
        assert_eq!(consumed, buf.len());
        assert_eq!(decoded.kind, FrameKind::Query);
        // First 16 bytes of payload = actor KAKI
        assert_eq!(&decoded.payload[..16], kaki.bytes());
        assert_eq!(&decoded.payload[16..], script);
    }

    #[test]
    fn encode_decode_result_end() {
        let frame = EnkiFrame::result_end(1_000_000_000, 487);
        let mut buf = Vec::new();
        FrameCodec::encode(&frame, &mut buf).unwrap();

        let (decoded, _) = FrameCodec::decode(&buf).unwrap().unwrap();
        assert_eq!(decoded.kind, FrameKind::ResultEnd);
        let total = u64::from_le_bytes(decoded.payload[..8].try_into().unwrap());
        let dur = u32::from_le_bytes(decoded.payload[8..12].try_into().unwrap());
        assert_eq!(total, 1_000_000_000);
        assert_eq!(dur, 487);
    }

    #[test]
    fn partial_data_returns_none() {
        let frame = EnkiFrame::ping();
        let mut buf = Vec::new();
        FrameCodec::encode(&frame, &mut buf).unwrap();
        // Only give 3 of 6 header bytes
        assert!(FrameCodec::decode(&buf[..3]).unwrap().is_none());
    }

    #[test]
    fn unknown_kind_is_error() {
        let bad = vec![0xFF, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(FrameCodec::decode(&bad).is_err());
    }

    #[test]
    fn ping_pong_roundtrip() {
        for frame in [EnkiFrame::ping(), EnkiFrame::pong()] {
            let mut buf = Vec::new();
            FrameCodec::encode(&frame, &mut buf).unwrap();
            let (d, _) = FrameCodec::decode(&buf).unwrap().unwrap();
            assert_eq!(d.kind, frame.kind);
        }
    }
}
