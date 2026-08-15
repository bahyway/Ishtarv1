#![forbid(unsafe_code)]
//! ReplicationEmitter — Write Pod side.
//!
//! Seals each `KakiSealedEvent` with KANĀKU (Ed25519), computes the chained
//! SHA3-256 digest, and appends the frame to the sovereign log file.
//!
//! The log file is opened with `O_APPEND` semantics — each `emit()` call
//! is atomic at the OS level for frames smaller than the filesystem block size
//! (~4 KiB).  For larger deltas, the frame_magic + frame_len header allows
//! the Broker to detect truncated writes.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use kupru::{AkkadianSeal, SealKeyPair};

use crate::error::{ReplicationError, ReplicationResult};
use crate::event::{KakiSealedEvent, ReplEventKind, GENESIS_DIGEST};

// ── ReplicationEmitter ────────────────────────────────────────────────────────

/// Owned by the Write Pod.  Holds the signing key; never exposes it outside.
pub struct ReplicationEmitter {
    write_pod_kaki_hash: u32,
    keypair:             SealKeyPair,
    log_path:            PathBuf,
    /// SHA3-256 digest of the last successfully written frame.
    last_digest:         [u8; 32],
    next_seq:            u64,
}

impl ReplicationEmitter {
    /// Create a new emitter.  `starting_seq` is 1 for a fresh log, or the
    /// last committed seq + 1 when resuming after a restart.
    pub fn new(
        write_pod_kaki_hash: u32,
        keypair:             SealKeyPair,
        log_path:            impl AsRef<Path>,
        starting_seq:        u64,
        last_digest:         [u8; 32],
    ) -> Self {
        ReplicationEmitter {
            write_pod_kaki_hash,
            keypair,
            log_path: log_path.as_ref().to_path_buf(),
            last_digest,
            next_seq: starting_seq,
        }
    }

    /// Convenience: create for a brand-new log (seq=1, genesis digest).
    pub fn new_log(
        write_pod_kaki_hash: u32,
        keypair:             SealKeyPair,
        log_path:            impl AsRef<Path>,
    ) -> Self {
        Self::new(write_pod_kaki_hash, keypair, log_path, 1, GENESIS_DIGEST)
    }

    // ── Core emit ─────────────────────────────────────────────────────────────

    /// Seal and append one event to the log.
    ///
    /// Steps:
    /// 1. Build `KakiSealedEvent` with current seq + prev_digest.
    /// 2. Compute canonical bytes (deterministic — see event.rs).
    /// 3. Sign with `SealKeyPair` (KANĀKU Ed25519).
    /// 4. Serialise to wire frame (includes SHA3-256 digest).
    /// 5. Append to log file (O_APPEND).
    /// 6. Advance seq and last_digest.
    pub fn emit(
        &mut self,
        kind:  ReplEventKind,
        delta: Vec<u8>,
        epoch: u32,
    ) -> ReplicationResult<KakiSealedEvent> {
        let mut ev = KakiSealedEvent::new(
            self.next_seq,
            epoch,
            self.write_pod_kaki_hash,
            self.last_digest,
            kind,
            delta,
        );

        // KANĀKU — Ed25519 sign canonical bytes
        let canonical = ev.canonical_bytes();
        let sig_bytes = self.keypair
            .seal(&canonical)
            .map_err(|e| ReplicationError::Io(format!("seal: {e}")))?;

        if sig_bytes.len() != 64 {
            return Err(ReplicationError::Io("Ed25519 signature must be 64 bytes".into()));
        }
        ev.seal.copy_from_slice(&sig_bytes);

        // Serialise to frame (computes ŠIPIR ŠARRI digest internally)
        let frame = ev.to_frame();

        // The last 32 bytes of the frame ARE the stored SHA3-256 digest.
        // Use this directly so emitter and broker agree on the chain value.
        let frame_digest: [u8; 32] = frame[frame.len()-32..].try_into()
            .map_err(|_| ReplicationError::Io("frame too short for digest".into()))?;
        ev.digest = frame_digest;

        // Append to log
        self.append_frame(&frame)?;

        // Advance state
        self.last_digest = frame_digest;
        self.next_seq   += 1;

        Ok(ev)
    }

    /// Current expected sequence number for the next event.
    pub fn next_seq(&self) -> u64 { self.next_seq }

    /// Digest of the last written frame.
    pub fn last_digest(&self) -> [u8; 32] { self.last_digest }

    // ── I/O ───────────────────────────────────────────────────────────────────

    fn append_frame(&self, frame: &[u8]) -> ReplicationResult<()> {
        let mut file: File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        file.write_all(frame)?;
        file.flush()?;
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{KakiSealedEvent, FRAME_MAGIC};

    fn temp_path() -> PathBuf {
        std::env::temp_dir().join(format!("enkwal_test_{}.enkwal",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()))
    }

    fn make_emitter(path: &PathBuf) -> ReplicationEmitter {
        let keypair = SealKeyPair::generate().expect("keypair");
        ReplicationEmitter::new_log(0xAAAA_0001, keypair, path)
    }

    #[test]
    fn emit_single_event() {
        let path = temp_path();
        let mut em = make_emitter(&path);
        let ev = em.emit(ReplEventKind::ParticleInsert, b"particle-data".to_vec(), 100_000)
            .expect("emit");
        assert_eq!(ev.seq, 1);
        assert_ne!(ev.seal, [0u8; 64]); // seal was set
        assert_ne!(ev.digest, [0u8; 32]); // digest was set
        assert_eq!(em.next_seq(), 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn emit_three_events_seq_increments() {
        let path = temp_path();
        let mut em = make_emitter(&path);
        for i in 1u64..=3 {
            let ev = em.emit(ReplEventKind::ParticleInsert, vec![i as u8; 4], 100_000 + i as u32)
                .expect("emit");
            assert_eq!(ev.seq, i);
        }
        assert_eq!(em.next_seq(), 4);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn log_file_starts_with_frame_magic() {
        let path = temp_path();
        let mut em = make_emitter(&path);
        em.emit(ReplEventKind::Checkpoint, b"chk".to_vec(), 1_000).expect("emit");
        let bytes = std::fs::read(&path).expect("read");
        assert!(bytes.starts_with(&FRAME_MAGIC));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn emitted_event_parses_back_correctly() {
        let path = temp_path();
        let mut em = make_emitter(&path);
        let ev = em.emit(ReplEventKind::ParticleUpdate, b"update-payload".to_vec(), 200_000)
            .expect("emit");
        let bytes = std::fs::read(&path).expect("read");
        let parsed = KakiSealedEvent::from_frame(&bytes).expect("parse");
        assert_eq!(parsed.seq,   ev.seq);
        assert_eq!(parsed.delta, ev.delta);
        assert_eq!(parsed.event_kind, ev.event_kind);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn chained_digest_links_events() {
        let path = temp_path();
        let mut em = make_emitter(&path);
        let ev1 = em.emit(ReplEventKind::ParticleInsert, b"first".to_vec(), 1_000).expect("emit");
        let ev2 = em.emit(ReplEventKind::ParticleInsert, b"second".to_vec(), 1_001).expect("emit");
        // Second event's prev_digest == first event's frame digest
        assert_eq!(ev2.prev_digest, ev1.digest);
        let _ = std::fs::remove_file(&path);
    }
}
