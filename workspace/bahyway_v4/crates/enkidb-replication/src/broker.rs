#![forbid(unsafe_code)]
//! ReplicationBroker — the sovereign 7-check verification gate.
//!
//! Runs with `--network none` in Podman.  Reads events from the Write Pod's
//! log (`:ro` volume mount), verifies all 7 layers, and writes verified events
//! to the Read Pod's input volume (`:rw` on the broker side).
//!
//! # Verification order (fail-fast — cheapest first)
//!
//! 1. Frame magic + structural integrity (fast byte check)
//! 2. Epoch freshness: event_epoch ≥ now − REPL_MAX_SECS
//! 3. Sequence monotonicity: seq == last_seq + 1
//! 4. Chained digest: event.prev_digest == broker.last_digest
//! 5. KANĀKU Ed25519 seal: verifier.verify(canonical_bytes, seal)
//! 6. ŠIPIR ŠARRI: SHA3-256(pre_digest_bytes) == event.digest (from_frame already checks this)
//! 7. HeptaSecSentinel: inspect_packet → must be Allow

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use hepta_sec_firewall::FirewallVerdict;
use hepta_sec_firewall::{KakiPacket, PacketProtocol};
use hepta_sec_sentinel::HeptaSecSentinel;
use kupru::{SealVerifier, SovereignVerifier};

use crate::error::{ReplicationError, ReplicationResult};
use crate::event::fnv1a_u32;
#[cfg(test)]
use crate::event::ReplEventKind;
use crate::event::{KakiSealedEvent, FRAME_MAGIC, GENESIS_DIGEST, REPL_MAX_SECS};

// ── PassportCheck ─────────────────────────────────────────────────────────────

/// Minimal passport check — scope + expiry.
/// Wraps the sovereign verification without importing the full SargonPassport
/// struct (which requires serde/uuid deps).  Callers provide a closure that
/// returns Ok(()) when the passport is valid.
pub type PassportValidator = Box<dyn Fn(u32) -> ReplicationResult<()> + Send + Sync>;

// ── BrokerConfig ──────────────────────────────────────────────────────────────

pub struct BrokerConfig {
    /// SHA3-256 verifying key bytes (32) from the Write Pod's SealKeyPair.
    pub write_pod_verifying_key: [u8; 32],
    /// KAKI hash of the Write Pod — must match every event's kaki_hash field.
    pub write_pod_kaki_hash: u32,
    /// Path to the Write Pod's append-only log (mounted `:ro`).
    pub input_log_path: PathBuf,
    /// Path to the verified output delta file (mounted `:rw` toward Read Pod).
    pub output_delta_path: PathBuf,
    /// Optional passport validator — called with `now_epoch` on each sweep.
    pub passport_validator: Option<PassportValidator>,
}

impl BrokerConfig {
    pub fn input_log_path(&self) -> &PathBuf {
        &self.input_log_path
    }
    pub fn output_delta_path(&self) -> &PathBuf {
        &self.output_delta_path
    }
}

// ── ReplicationBroker ─────────────────────────────────────────────────────────

/// Sovereign 7-layer replication broker.
pub struct ReplicationBroker {
    config: BrokerConfig,
    verifier: SovereignVerifier,
    sentinel: HeptaSecSentinel,
    last_seq: u64,
    last_digest: [u8; 32],
    /// Bytes consumed from the input log so far (file cursor position).
    log_offset: u64,
    /// Total events successfully forwarded to the Read Pod.
    forwarded: u64,
    /// Total events rejected (any check failed).
    rejected: u64,
}

impl ReplicationBroker {
    pub fn new(config: BrokerConfig) -> Self {
        let verifier = SovereignVerifier::from_bytes(&config.write_pod_verifying_key)
            .expect("invalid Write Pod verifying key");

        let mut sentinel = HeptaSecSentinel::new();
        // Pre-register Write Pod's KAKI at maximum B11 score
        sentinel.register_kaki_b11(config.write_pod_kaki_hash, 255, 0);

        ReplicationBroker {
            config,
            verifier,
            sentinel,
            last_seq: 0,
            last_digest: GENESIS_DIGEST,
            log_offset: 0,
            forwarded: 0,
            rejected: 0,
        }
    }

    // ── Main sweep ────────────────────────────────────────────────────────────

    /// Process all unread frames from the input log.
    ///
    /// Reads new frames since `log_offset`, runs 7-layer verification on each,
    /// and appends verified frames to the output delta file.
    ///
    /// Returns the number of events forwarded in this sweep.
    pub fn sweep(&mut self, now_epoch: u32) -> ReplicationResult<usize> {
        let new_bytes = self.read_new_log_bytes()?;
        if new_bytes.is_empty() {
            return Ok(0);
        }

        let mut forwarded_this_sweep = 0usize;
        let mut pos = 0usize;

        while pos < new_bytes.len() {
            // Need at least 8 bytes (magic + frame_len)
            if pos + 8 > new_bytes.len() {
                break;
            }
            if new_bytes[pos..pos + 4] != FRAME_MAGIC {
                return Err(ReplicationError::InvalidFrame);
            }
            let frame_len =
                u32::from_le_bytes(new_bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
            let total = 8 + frame_len;
            if pos + total > new_bytes.len() {
                // Partial write — stop here, retry next sweep
                break;
            }
            let frame = &new_bytes[pos..pos + total];

            match self.verify_and_forward(frame, now_epoch) {
                Ok(ev) => {
                    self.last_seq = ev.seq;
                    self.last_digest = ev.digest;
                    self.log_offset += total as u64;
                    self.forwarded += 1;
                    forwarded_this_sweep += 1;
                }
                Err(e) => {
                    self.rejected += 1;
                    return Err(e);
                }
            }
            pos += total;
        }
        Ok(forwarded_this_sweep)
    }

    // ── 7-layer verification ──────────────────────────────────────────────────

    fn verify_and_forward(
        &mut self,
        frame: &[u8],
        now_epoch: u32,
    ) -> ReplicationResult<KakiSealedEvent> {
        // ── Check 1+6: frame integrity + SHA3-256 ŠIPIR ŠARRI ────────────────
        // from_frame() validates frame magic, digest, and payload_fnv together.
        let ev = KakiSealedEvent::from_frame(frame)?;

        // ── Check 2: epoch freshness ──────────────────────────────────────────
        let age = now_epoch.saturating_sub(ev.epoch);
        if age > REPL_MAX_SECS {
            return Err(ReplicationError::EpochExpired {
                event_epoch: ev.epoch,
                now_epoch,
            });
        }

        // ── Check 3: sequence monotonicity ───────────────────────────────────
        let expected_seq = self.last_seq + 1;
        if ev.seq != expected_seq {
            return Err(ReplicationError::SequenceGap {
                expected: expected_seq,
                got: ev.seq,
            });
        }

        // ── Check 4: chained digest ───────────────────────────────────────────
        use subtle::ConstantTimeEq;
        if ev.prev_digest.ct_eq(&self.last_digest).unwrap_u8() == 0 {
            return Err(ReplicationError::ChainBroken);
        }

        // ── Check 5: KAKI hash matches configured Write Pod ───────────────────
        if ev.write_pod_kaki_hash != self.config.write_pod_kaki_hash {
            return Err(ReplicationError::KakiBlocked);
        }

        // ── Check 5b: KANĀKU Ed25519 seal ─────────────────────────────────────
        let canonical = ev.canonical_bytes();
        if !self.verifier.verify(&canonical, &ev.seal) {
            return Err(ReplicationError::SealInvalid);
        }

        // ── Check 7: HeptaSecSentinel ─────────────────────────────────────────
        let packet = KakiPacket {
            src_kaki_hash: Some(ev.write_pod_kaki_hash),
            dst_kaki_hash: Some(fnv1a_u32(b"ReadPodKaki")),
            payload_fnv: ev.payload_fnv,
            protocol: PacketProtocol::KakiNative,
            epoch: ev.epoch,
            size_bytes: ev.delta.len() as u32,
        };
        match self.sentinel.inspect_packet(&packet, now_epoch) {
            FirewallVerdict::Allow { .. } => {}
            _ => return Err(ReplicationError::KakiBlocked),
        }

        // ── Optional: SargonPassport validator ───────────────────────────────
        if let Some(ref validator) = self.config.passport_validator {
            validator(now_epoch)?;
        }

        // ── All 7 checks passed — forward to output ───────────────────────────
        self.append_verified_frame(frame)?;

        Ok(ev)
    }

    // ── I/O ───────────────────────────────────────────────────────────────────

    fn read_new_log_bytes(&self) -> ReplicationResult<Vec<u8>> {
        let mut file = File::open(&self.config.input_log_path)
            .map_err(|e| ReplicationError::Io(e.to_string()))?;

        use std::io::Seek;
        file.seek(std::io::SeekFrom::Start(self.log_offset))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn append_verified_frame(&self, frame: &[u8]) -> ReplicationResult<()> {
        let mut file: File = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.output_delta_path)?;
        file.write_all(frame)?;
        file.flush()?;
        Ok(())
    }

    // ── Stats ─────────────────────────────────────────────────────────────────

    pub fn forwarded(&self) -> u64 {
        self.forwarded
    }
    pub fn rejected(&self) -> u64 {
        self.rejected
    }
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }
    pub fn log_offset(&self) -> u64 {
        self.log_offset
    }
    pub fn input_log_path(&self) -> &PathBuf {
        &self.config.input_log_path
    }
    pub fn output_delta_path(&self) -> &PathBuf {
        &self.config.output_delta_path
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emitter::ReplicationEmitter;
    use kupru::SealKeyPair;

    fn temp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "enkwal_broker_{}_{}.enkwal",
            tag,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ))
    }

    fn make_pair(kaki_hash: u32) -> (ReplicationEmitter, ReplicationBroker) {
        let keypair = SealKeyPair::generate().expect("keypair");
        let verifying_key = keypair.verifying_key_bytes();
        let input = temp_path("input");
        let output = temp_path("output");
        let emitter = ReplicationEmitter::new_log(kaki_hash, keypair, &input);
        let config = BrokerConfig {
            write_pod_verifying_key: verifying_key,
            write_pod_kaki_hash: kaki_hash,
            input_log_path: input,
            output_delta_path: output,
            passport_validator: None,
        };
        let broker = ReplicationBroker::new(config);
        (emitter, broker)
    }

    fn cleanup(_em: &ReplicationEmitter, br: &ReplicationBroker) {
        let _ = std::fs::remove_file(br.input_log_path());
        let _ = std::fs::remove_file(br.output_delta_path());
    }

    #[test]
    fn single_event_end_to_end() {
        let (mut em, mut br) = make_pair(0xAAAA_0001);
        em.emit(ReplEventKind::ParticleInsert, b"data".to_vec(), 1_000)
            .unwrap();
        let n = br.sweep(1_000).unwrap();
        assert_eq!(n, 1);
        assert_eq!(br.forwarded(), 1);
        assert_eq!(br.last_seq(), 1);
        cleanup(&em, &br);
    }

    #[test]
    fn three_events_forwarded() {
        let (mut em, mut br) = make_pair(0xBBBB_0002);
        for i in 0..3u32 {
            em.emit(ReplEventKind::ParticleInsert, vec![i as u8], 1_000 + i)
                .unwrap();
        }
        let n = br.sweep(1_002).unwrap();
        assert_eq!(n, 3);
        assert_eq!(br.forwarded(), 3);
        cleanup(&em, &br);
    }

    #[test]
    fn expired_epoch_rejected() {
        let (mut em, mut br) = make_pair(0xCCCC_0003);
        em.emit(ReplEventKind::ParticleInsert, b"old".to_vec(), 1_000)
            .unwrap();
        // now_epoch is REPL_MAX_SECS + 1 seconds after event epoch
        let result = br.sweep(1_000 + REPL_MAX_SECS + 1);
        assert!(matches!(result, Err(ReplicationError::EpochExpired { .. })));
        cleanup(&em, &br);
    }

    #[test]
    fn wrong_kaki_hash_rejected() {
        let keypair = SealKeyPair::generate().unwrap();
        let verifying_key = keypair.verifying_key_bytes();
        let input = temp_path("kaki_wrong_in");
        let output = temp_path("kaki_wrong_out");

        // Emitter uses kaki_hash 0x1111, broker expects 0x2222
        let mut em = ReplicationEmitter::new_log(0x1111_0001, keypair, &input);
        let config = BrokerConfig {
            write_pod_verifying_key: verifying_key,
            write_pod_kaki_hash: 0x2222_0001, // mismatch
            input_log_path: input.clone(),
            output_delta_path: output.clone(),
            passport_validator: None,
        };
        let mut br = ReplicationBroker::new(config);
        em.emit(ReplEventKind::ParticleInsert, b"data".to_vec(), 1_000)
            .unwrap();
        assert!(matches!(
            br.sweep(1_000),
            Err(ReplicationError::KakiBlocked)
        ));
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn output_file_contains_verified_frame() {
        let (mut em, mut br) = make_pair(0xDDDD_0004);
        em.emit(ReplEventKind::ParticleUpdate, b"payload".to_vec(), 5_000)
            .unwrap();
        br.sweep(5_000).unwrap();
        let out = std::fs::read(br.output_delta_path()).unwrap();
        assert!(out.starts_with(&FRAME_MAGIC));
        cleanup(&em, &br);
    }

    #[test]
    fn second_sweep_skips_already_forwarded() {
        let (mut em, mut br) = make_pair(0xEEEE_0005);
        em.emit(ReplEventKind::ParticleInsert, b"ev1".to_vec(), 1_000)
            .unwrap();
        br.sweep(1_000).unwrap();
        // Second sweep with no new events
        let n = br.sweep(1_001).unwrap();
        assert_eq!(n, 0);
        assert_eq!(br.forwarded(), 1);
        cleanup(&em, &br);
    }
}
