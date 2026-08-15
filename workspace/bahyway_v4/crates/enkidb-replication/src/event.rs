#![forbid(unsafe_code)]
//! KakiSealedEvent — sovereign replication unit.
//!
//! Wire format (all integers little-endian):
//!
//! ```text
//! [4]  FRAME_MAGIC   0xEE 0x4B 0x57 0x04
//! [4]  frame_len     u32 LE  (all bytes that follow, including seal+digest)
//! [8]  seq           u64 LE  (monotonically increasing from 1)
//! [4]  epoch         u32 LE  (seconds since sovereign epoch)
//! [4]  write_pod_kaki_hash  u32 LE
//! [32] prev_digest   [u8;32] (SHA3-256 of previous event frame; genesis=all-zero)
//! [4]  payload_fnv   u32 LE  (FNV-1a fast check of delta bytes)
//! [1]  event_kind    u8
//! [4]  delta_len     u32 LE
//! [N]  delta         [u8; delta_len]
//! [64] seal          [u8;64] Ed25519 over canonical bytes
//! [32] digest        [u8;32] SHA3-256 of all bytes before this field
//! ```
//!
//! Canonical bytes (signed by KANĀKU):
//!   REPL_DOMAIN || seq || epoch || kaki_hash || prev_digest || kind || delta

use sha3::{Digest, Sha3_256};

use crate::error::{ReplicationError, ReplicationResult};

/// Frame magic — "ENKWAL v4"
pub const FRAME_MAGIC: [u8; 4] = [0xEE, 0x4B, 0x57, 0x04];

/// Domain separator for replication canonical bytes.
pub const REPL_DOMAIN: &[u8] = b"BahyWay.Replication.v4.ENKWAL";

/// Events older than this are rejected by the Broker. 5 minutes.
pub const REPL_MAX_SECS: u32 = 300;

/// All-zero digest used as `prev_digest` for the very first event.
pub const GENESIS_DIGEST: [u8; 32] = [0u8; 32];

// ── ReplEventKind ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplEventKind {
    ParticleInsert  = 0,
    ParticleUpdate  = 1,
    ParticleDelete  = 2,
    /// Periodic full-state snapshot marker — delta contains a checkpoint token.
    Checkpoint      = 3,
}

impl ReplEventKind {
    pub fn as_u8(self) -> u8 { self as u8 }

    pub fn from_u8(v: u8) -> ReplicationResult<Self> {
        match v {
            0 => Ok(ReplEventKind::ParticleInsert),
            1 => Ok(ReplEventKind::ParticleUpdate),
            2 => Ok(ReplEventKind::ParticleDelete),
            3 => Ok(ReplEventKind::Checkpoint),
            _ => Err(ReplicationError::InvalidFrame),
        }
    }
}

// ── KakiSealedEvent ───────────────────────────────────────────────────────────

/// One sovereign replication event — KANĀKU-sealed and chain-hashed.
#[derive(Debug, Clone)]
pub struct KakiSealedEvent {
    pub seq:                  u64,
    pub epoch:                u32,
    pub write_pod_kaki_hash:  u32,
    /// SHA3-256 of the previous event's full frame bytes (up to but not
    /// including that event's own `digest` field).
    /// First event uses `GENESIS_DIGEST` (all zeros).
    pub prev_digest:          [u8; 32],
    /// FNV-1a of delta bytes — fast corruption check before Ed25519.
    pub payload_fnv:          u32,
    pub event_kind:           ReplEventKind,
    pub delta:                Vec<u8>,
    /// Ed25519 signature over `canonical_bytes()`.  Zero before sealing.
    pub seal:                 [u8; 64],
    /// SHA3-256 of the full frame up to (not including) this field.
    /// Zero before finalisation.
    pub digest:               [u8; 32],
}

impl KakiSealedEvent {
    /// Construct an unsigned, undigested event.  Call `emitter.emit()` instead
    /// of this directly — it sets `seal` and `digest`.
    pub fn new(
        seq:                 u64,
        epoch:               u32,
        write_pod_kaki_hash: u32,
        prev_digest:         [u8; 32],
        event_kind:          ReplEventKind,
        delta:               Vec<u8>,
    ) -> Self {
        let payload_fnv = fnv1a_u32(&delta);
        KakiSealedEvent {
            seq, epoch, write_pod_kaki_hash, prev_digest,
            payload_fnv, event_kind, delta,
            seal:   [0u8; 64],
            digest: [0u8; 32],
        }
    }

    /// Canonical bytes — what the Ed25519 KANĀKU seal is computed over.
    /// Domain-separated, deterministic, no seal/digest fields included.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(REPL_DOMAIN.len() + 4 + 8+4+4+32+4+1+4 + self.delta.len());
        push_bytes_lp(&mut v, REPL_DOMAIN);
        v.extend_from_slice(&self.seq.to_le_bytes());
        v.extend_from_slice(&self.epoch.to_le_bytes());
        v.extend_from_slice(&self.write_pod_kaki_hash.to_le_bytes());
        v.extend_from_slice(&self.prev_digest);
        v.push(self.event_kind.as_u8());
        v.extend_from_slice(&(self.delta.len() as u32).to_le_bytes());
        v.extend_from_slice(&self.delta);
        v
    }

    /// Serialise to the .enkwal wire format.
    /// `seal` and `digest` must be set before calling this.
    pub fn to_frame(&self) -> Vec<u8> {
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(&self.seq.to_le_bytes());
        body.extend_from_slice(&self.epoch.to_le_bytes());
        body.extend_from_slice(&self.write_pod_kaki_hash.to_le_bytes());
        body.extend_from_slice(&self.prev_digest);
        body.extend_from_slice(&self.payload_fnv.to_le_bytes());
        body.push(self.event_kind.as_u8());
        body.extend_from_slice(&(self.delta.len() as u32).to_le_bytes());
        body.extend_from_slice(&self.delta);
        body.extend_from_slice(&self.seal);
        // digest = SHA3-256 of (FRAME_MAGIC || frame_len || body)
        // We compute it here: frame so far = MAGIC(4) + len(4) + body
        let frame_len = body.len() as u32 + 32; // +32 for digest itself
        let mut pre_digest: Vec<u8> = Vec::new();
        pre_digest.extend_from_slice(&FRAME_MAGIC);
        pre_digest.extend_from_slice(&frame_len.to_le_bytes());
        pre_digest.extend_from_slice(&body);
        let computed_digest: [u8; 32] = Sha3_256::digest(&pre_digest).into();

        // Final frame
        let mut frame: Vec<u8> = Vec::with_capacity(8 + body.len() + 32);
        frame.extend_from_slice(&FRAME_MAGIC);
        frame.extend_from_slice(&frame_len.to_le_bytes());
        frame.extend_from_slice(&body);
        frame.extend_from_slice(&computed_digest);
        frame
    }

    /// Parse one event from a frame byte slice.
    pub fn from_frame(frame: &[u8]) -> ReplicationResult<Self> {
        if frame.len() < 8 {
            return Err(ReplicationError::InvalidFrame);
        }
        if &frame[0..4] != &FRAME_MAGIC {
            return Err(ReplicationError::InvalidFrame);
        }
        let frame_len = u32::from_le_bytes(frame[4..8].try_into().unwrap()) as usize;
        if frame.len() < 8 + frame_len {
            return Err(ReplicationError::InvalidFrame);
        }
        let body_with_digest = &frame[8..8 + frame_len];
        if body_with_digest.len() < 32 {
            return Err(ReplicationError::InvalidFrame);
        }

        // Last 32 bytes = stored digest
        let stored_digest: [u8; 32] = body_with_digest[body_with_digest.len()-32..]
            .try_into().map_err(|_| ReplicationError::InvalidFrame)?;

        // Verify digest = SHA3-256(magic || frame_len_bytes || body_without_digest)
        let pre_digest_len = 4 + 4 + (body_with_digest.len() - 32);
        let mut pre_digest = Vec::with_capacity(pre_digest_len);
        pre_digest.extend_from_slice(&FRAME_MAGIC);
        pre_digest.extend_from_slice(&(frame_len as u32).to_le_bytes());
        pre_digest.extend_from_slice(&body_with_digest[..body_with_digest.len()-32]);
        let computed: [u8; 32] = Sha3_256::digest(&pre_digest).into();

        use subtle::ConstantTimeEq;
        if computed.ct_eq(&stored_digest).unwrap_u8() == 0 {
            return Err(ReplicationError::DigestMismatch);
        }

        // Parse fields from body (without the trailing digest)
        let body = &body_with_digest[..body_with_digest.len()-32];
        let mut pos = 0usize;

        macro_rules! read_fixed {
            ($n:expr) => {{
                if pos + $n > body.len() { return Err(ReplicationError::InvalidFrame); }
                let s = &body[pos..pos+$n];
                pos += $n;
                #[allow(unused_assignments)]
                { let _ = pos; }
                s
            }};
        }

        let seq   = u64::from_le_bytes(read_fixed!(8).try_into().unwrap());
        let epoch = u32::from_le_bytes(read_fixed!(4).try_into().unwrap());
        let write_pod_kaki_hash = u32::from_le_bytes(read_fixed!(4).try_into().unwrap());
        let prev_digest: [u8; 32] = read_fixed!(32).try_into().unwrap();
        let payload_fnv = u32::from_le_bytes(read_fixed!(4).try_into().unwrap());
        let event_kind = ReplEventKind::from_u8(read_fixed!(1)[0])?;
        let delta_len  = u32::from_le_bytes(read_fixed!(4).try_into().unwrap()) as usize;
        let delta      = read_fixed!(delta_len).to_vec();
        let seal: [u8; 64] = read_fixed!(64).try_into().map_err(|_| ReplicationError::InvalidFrame)?;

        // Verify payload_fnv
        let expected_fnv = fnv1a_u32(&delta);
        if payload_fnv != expected_fnv {
            return Err(ReplicationError::DigestMismatch);
        }

        Ok(KakiSealedEvent {
            seq, epoch, write_pod_kaki_hash, prev_digest,
            payload_fnv, event_kind, delta, seal,
            digest: stored_digest,
        })
    }

    /// Compute the digest of this event's frame bytes
    /// (what the next event stores as its `prev_digest`).
    pub fn frame_digest(frame_bytes: &[u8]) -> [u8; 32] {
        Sha3_256::digest(frame_bytes).into()
    }
}

// ── FNV-1a ────────────────────────────────────────────────────────────────────

pub(crate) fn fnv1a_u32(data: &[u8]) -> u32 {
    const OFFSET: u32 = 2_166_136_261;
    const PRIME:  u32 = 16_777_619;
    let mut h = OFFSET;
    for b in data {
        h ^= *b as u32;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn push_bytes_lp(v: &mut Vec<u8>, data: &[u8]) {
    v.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v.extend_from_slice(data);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(seq: u64, prev: [u8; 32]) -> KakiSealedEvent {
        KakiSealedEvent::new(seq, 100_000, 0xAAAA_0001, prev, ReplEventKind::ParticleInsert, b"test-delta".to_vec())
    }

    #[test]
    fn event_kind_roundtrip() {
        for kind in [ReplEventKind::ParticleInsert, ReplEventKind::ParticleUpdate,
                     ReplEventKind::ParticleDelete, ReplEventKind::Checkpoint] {
            assert_eq!(ReplEventKind::from_u8(kind.as_u8()).unwrap(), kind);
        }
    }

    #[test]
    fn canonical_bytes_deterministic() {
        let ev = sample_event(1, GENESIS_DIGEST);
        assert_eq!(ev.canonical_bytes(), ev.canonical_bytes());
    }

    #[test]
    fn frame_roundtrip_requires_seal_and_digest() {
        let mut ev = sample_event(1, GENESIS_DIGEST);
        // Set dummy seal (all zeros default) — frame/parse should work
        let frame = ev.to_frame();
        assert!(frame.starts_with(&FRAME_MAGIC));

        let parsed = KakiSealedEvent::from_frame(&frame).unwrap();
        assert_eq!(parsed.seq,   ev.seq);
        assert_eq!(parsed.epoch, ev.epoch);
        assert_eq!(parsed.delta, ev.delta);
        assert_eq!(parsed.prev_digest, ev.prev_digest);
    }

    #[test]
    fn tampered_payload_fails_digest() {
        let ev = sample_event(1, GENESIS_DIGEST);
        let mut frame = ev.to_frame();
        // Flip a byte in the delta region
        let delta_pos = 4 + 4 + 8+4+4+32+4+1+4; // past fixed fields
        frame[delta_pos] ^= 0xFF;
        assert!(matches!(KakiSealedEvent::from_frame(&frame), Err(ReplicationError::DigestMismatch)));
    }

    #[test]
    fn wrong_magic_fails() {
        let ev = sample_event(1, GENESIS_DIGEST);
        let mut frame = ev.to_frame();
        frame[0] = 0x00;
        assert!(matches!(KakiSealedEvent::from_frame(&frame), Err(ReplicationError::InvalidFrame)));
    }

    #[test]
    fn genesis_digest_is_all_zeros() {
        assert_eq!(GENESIS_DIGEST, [0u8; 32]);
    }

    #[test]
    fn payload_fnv_mismatch_fails() {
        let ev = sample_event(1, GENESIS_DIGEST);
        let frame = ev.to_frame();
        // Corrupt the payload_fnv field (bytes 52..56 in body = magic(4)+len(4)+seq(8)+epoch(4)+kaki(4)+prev(32))
        let fnv_offset = 4 + 4 + 8 + 4 + 4 + 32;
        let mut frame2 = frame.clone();
        frame2[fnv_offset] ^= 0x01;
        // This corrupts the digest too → DigestMismatch
        assert!(KakiSealedEvent::from_frame(&frame2).is_err());
    }
}
