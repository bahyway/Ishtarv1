#![forbid(unsafe_code)]
//! journal_bridge — wires `enkidb-replication`'s real KAKI-sealed pipeline
//! to a real `enkidb_journal::Journal`, and from there to real materialized
//! Data Files (§9 — this closes the exact gap `enkiddb-write-server` and
//! `enkimdb-write-server`'s own module doc comments flag: "the real
//! durability fix, not wired in here").
//!
//! ## Delta encoding
//! `ReplEventKind::ParticleInsert`/`ParticleUpdate`'s `delta` bytes are a
//! `JournalEntry::to_bytes()` encoding (`enkidb-journal`'s own real,
//! already-tested 40-byte-header + EAV-triples wire format — the same one
//! `enkidb-persist::disk_journal` uses for its on-disk WAL). No JSON, no
//! new format invented for this: one real `JournalEntry` encoding, reused.
//! `ParticleDelete` and `Checkpoint` are accepted by `ReplEventKind` but
//! have no real caller anywhere in this codebase yet and are rejected here
//! with `DeltaDecodeFailed` rather than silently no-op'd — a future
//! extension, not pretended to already exist.
//!
//! ## Honest note on the Broker's key
//! `ReplicationBroker::append_verified_frame` forwards the verified frame
//! byte-for-byte — it does not re-sign it with a Broker-owned key (there
//! is no such key anywhere in `BrokerConfig`). So the seal a downstream
//! `ReplicationConsumer` checks is still, in practice, the original Write
//! Pod's seal. `apply_delta_to_journal` below is agnostic to this (it only
//! runs after `ReplicationConsumer::verify_event` already accepted the
//! frame) — callers constructing a `ReplicationConsumer` for real use
//! today should pass the Write Pod's own verifying key as the "broker
//! verifying key" parameter, since that is what the seal actually is. A
//! real Broker-resign step is a separate, not-yet-built hardening step,
//! flagged here rather than pretended away.
use enkidb_journal::entry::JournalEntry;
use enkidb_journal::Journal;
use enkidb_readnode::materialize::{materialize, MaterializeStats};

use crate::error::{ReplicationError, ReplicationResult};
use crate::event::ReplEventKind;
use crate::emitter::ReplicationEmitter;

/// Encode `entry` as replication delta bytes and emit it as `kind`
/// (`ParticleInsert` for a brand-new particle, `ParticleUpdate` for a
/// further event against an already-known one — the distinction is the
/// caller's, `JournalEntry` itself carries no "is this the first event"
/// flag).
///
/// `now_epoch` is the replication frame's own freshness epoch (real
/// wall-clock seconds — the Broker rejects anything more than
/// `REPL_MAX_SECS` stale, see `event::REPL_MAX_SECS`). This is
/// deliberately a *separate* parameter from `entry.epoch`, which is
/// `JournalEntry`'s own logical partition key (small sequential integers,
/// §9.1) — conflating the two was a real bug caught by this module's own
/// integration test: reusing `entry.epoch` as the replication epoch made
/// every real-world event look hundreds of "seconds" stale to the
/// freshness check on its very first sweep.
pub fn emit_journal_entry(
    emitter:   &mut ReplicationEmitter,
    kind:      ReplEventKind,
    entry:     &JournalEntry,
    now_epoch: u32,
) -> ReplicationResult<()> {
    emitter.emit(kind, entry.to_bytes(), now_epoch)?;
    Ok(())
}

/// Apply one verified event's delta to `journal` — the callback shape
/// `ReplicationConsumer::new`'s `ApplyFn` expects (`FnMut(u8, &[u8])`).
/// `ParticleInsert`/`ParticleUpdate` decode via `JournalEntry::from_bytes`
/// and append to `journal`. `ParticleDelete`/`Checkpoint` have no real
/// handling yet -- rejected, not silently accepted.
pub fn apply_delta_to_journal(journal: &mut Journal, kind: u8, delta: &[u8]) -> ReplicationResult<()> {
    let event_kind = ReplEventKind::from_u8(kind)?;
    match event_kind {
        ReplEventKind::ParticleInsert | ReplEventKind::ParticleUpdate => {
            let (entry, _consumed) = JournalEntry::from_bytes(delta)
                .ok_or(ReplicationError::DeltaDecodeFailed)?;
            journal.append(entry).map_err(|e| ReplicationError::Io(e.to_string()))
        }
        ReplEventKind::ParticleDelete | ReplEventKind::Checkpoint => {
            Err(ReplicationError::DeltaDecodeFailed)
        }
    }
}

/// Materialize `journal`'s current state into real Data Files at
/// `entities_base`/`eav_index_base` -- the same `enkidb_readnode::
/// materialize` every other EnkiDB Type's Write Node uses, so a Read Node
/// fed purely through replication (no direct Journal access of its own)
/// still ends up served by the exact same `CachedReadNode` path.
pub fn materialize_journal(
    journal:        &Journal,
    entities_base:  impl AsRef<std::path::Path>,
    eav_index_base: impl AsRef<std::path::Path>,
) -> std::io::Result<MaterializeStats> {
    materialize(journal, entities_base, eav_index_base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::{BrokerConfig, ReplicationBroker};
    use crate::consumer::ReplicationConsumer;
    use crate::event::fnv1a_u32;
    use bahyway_core::TribeId;
    use enkidb_journal::entry::EavTriple;
    use enkidb_kaki::{EventKaki, IdentityKaki, KakiMinter, KakiRole};
    use enkidb_readnode::CachedReadNode;
    use kupru::SealKeyPair;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    fn tmp_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "enkwal_bridge_{tag}_{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().subsec_nanos()
        ))
    }

    fn make_entry(minter: &KakiMinter, epoch: u32, state: &str) -> JournalEntry {
        use akkvalue::{codec, AkkValue};
        let ek = EventKaki::try_from_kaki(minter.event(KakiRole::Zikru)).unwrap();
        let ik = IdentityKaki::try_from_kaki(minter.identity(KakiRole::Zikru)).unwrap();
        JournalEntry::new(ek, ik, epoch, vec![
            EavTriple::new(
                bahyway_crc::crc16("station".as_bytes()) as u32,
                codec::encode(&AkkValue::Text(state.to_string())),
            ),
        ])
    }

    /// Full real pipeline: JournalEntry -> emit -> broker.sweep -> consumer.
    /// consume(apply_fn = append to a Journal) -> materialize that Journal
    /// -> CachedReadNode query finds the replicated particle. This is the
    /// real evidence for "wired into Data Files", not just a claim.
    #[test]
    fn replicated_entry_is_queryable_after_materialize() {
        let tribe  = TribeId::from_u16(0x7A00);
        let minter = KakiMinter::new(tribe);
        let entry  = make_entry(&minter, 1, "replicated-station");

        let write_kp = SealKeyPair::generate().unwrap();
        let write_vk = write_kp.verifying_key_bytes();
        let kaki_hash = 0xB01D_0001u32;

        let log_path   = tmp_path("log");
        let delta_path = tmp_path("delta");

        let mut emitter = ReplicationEmitter::new_log(kaki_hash, write_kp, &log_path);
        emit_journal_entry(&mut emitter, ReplEventKind::ParticleInsert, &entry, 1_000).unwrap();

        let mut broker = ReplicationBroker::new(BrokerConfig {
            write_pod_verifying_key: write_vk,
            write_pod_kaki_hash:     kaki_hash,
            input_log_path:          log_path.clone(),
            output_delta_path:       delta_path.clone(),
            passport_validator:      None,
        });
        let forwarded = broker.sweep(1_000).unwrap();
        assert_eq!(forwarded, 1);

        // Per this module's own honest note above: the frame's seal really
        // is the Write Pod's seal (the Broker doesn't re-sign), so the
        // Consumer is constructed with the Write Pod's verifying key.
        let read_journal: Arc<Mutex<Journal>> = Arc::new(Mutex::new(Journal::new(64)));
        let apply_journal = Arc::clone(&read_journal);
        let mut consumer = ReplicationConsumer::new(
            write_vk,
            kaki_hash,
            &delta_path,
            Box::new(move |kind, delta| {
                let mut j = apply_journal.lock().unwrap();
                apply_delta_to_journal(&mut j, kind, delta)
            }),
        );
        let applied = consumer.consume(1_000).unwrap();
        assert_eq!(applied, 1);
        assert_eq!(consumer.stats().events_applied, 1);

        // Now the real materialize + CachedReadNode path. `materialize`
        // (like every other Write Node in this fleet) always APPENDS --
        // `enkidb_datafile::DataFileWriter::open` never truncates -- so a
        // leftover file at a reused path would silently duplicate/shadow
        // records instead of failing loudly. Fresh directory, wiped before
        // writing, same as every real *-write-server's materialize_fresh.
        let current = tmp_path("materialized_dir");
        let _ = fs::remove_dir_all(&current);
        fs::create_dir_all(&current).unwrap();
        let entities = current.join("entities");
        let eav      = current.join("eav");
        {
            let j = read_journal.lock().unwrap();
            let stats = materialize_journal(&j, &entities, &eav).unwrap();
            assert_eq!(stats.entities, 1);
        }

        let crn = CachedReadNode::open(&entities, &eav).unwrap();
        let result = crn.query("WHO T.E\nWHERE E[station] = \"replicated-station\"").unwrap();
        assert_eq!(result.matched.len(), 1, "the replicated particle must be queryable after materialize");
        assert_eq!(result.matched[0].entity.bytes(), entry.target_kaki.bytes());

        let _ = fs::remove_file(&log_path);
        let _ = fs::remove_file(&delta_path);
        let _ = fs::remove_dir_all(&current);
    }

    #[test]
    fn apply_delta_to_journal_rejects_undecodable_bytes() {
        let mut journal = Journal::new(64);
        let err = apply_delta_to_journal(&mut journal, ReplEventKind::ParticleInsert.as_u8(), b"not-a-journal-entry");
        assert!(matches!(err, Err(ReplicationError::DeltaDecodeFailed)));
    }

    #[test]
    fn apply_delta_to_journal_rejects_delete_and_checkpoint_for_now() {
        let mut journal = Journal::new(64);
        assert!(matches!(
            apply_delta_to_journal(&mut journal, ReplEventKind::ParticleDelete.as_u8(), b""),
            Err(ReplicationError::DeltaDecodeFailed)
        ));
        assert!(matches!(
            apply_delta_to_journal(&mut journal, ReplEventKind::Checkpoint.as_u8(), b""),
            Err(ReplicationError::DeltaDecodeFailed)
        ));
    }

    #[test]
    fn emit_journal_entry_round_trips_through_to_bytes() {
        let minter = KakiMinter::new(TribeId::from_u16(0x7A01));
        let entry  = make_entry(&minter, 5, "round-trip-station");
        let kp = SealKeyPair::generate().unwrap();
        let log_path = tmp_path("emit_only");
        let mut emitter = ReplicationEmitter::new_log(0xCAFE_0001, kp, &log_path);

        emit_journal_entry(&mut emitter, ReplEventKind::ParticleInsert, &entry, 5_000).unwrap();

        let raw = fs::read(&log_path).unwrap();
        let ev  = crate::event::KakiSealedEvent::from_frame(&raw).unwrap();
        assert_eq!(ev.epoch, 5_000, "replication frame epoch is the freshness epoch, not entry.epoch");
        let (decoded, _) = JournalEntry::from_bytes(&ev.delta).unwrap();
        assert_eq!(decoded.epoch, 5, "JournalEntry's own logical epoch must survive the round trip");
        let (decoded_value, _) = akkvalue::codec::decode(&decoded.eav[0].value, 0).unwrap();
        assert_eq!(decoded_value, akkvalue::AkkValue::Text("round-trip-station".to_string()));

        let _ = fs::remove_file(&log_path);
        let _ = fnv1a_u32(b"unused"); // keep import honest if unused elsewhere
    }
}
